//! Transcoding and preservation of metadata across formats

use std::fmt::Display;
use std::iter::zip;
use std::process::Command;
use std::process::Stdio;

use anyhow::Context;
use anyhow::Result;
use id3::TagLike;
use lofty::AudioFile;
use lofty::ParseOptions;
use ratatui::widgets::ListItem;
use walkdir::DirEntry;
use walkdir::WalkDir;

use crate::io::Sort;
use crate::io::Walk;
use crate::release::Release;

// transcoding
// flac
// https://github.com/ruuda/claxon
// https://github.com/ruuda/claxon/blob/master/examples/decode_simple.rs -- i like the idea of
// transcoding sample-by-sample; as long as mp3s also have a raw byte array
// (&[u16]) representation

// lame vbr is not very well documented; under the hood, v0 is 'preset mode
// 500', whatever that means...
// https://lame.sourceforge.io/vbr.php
// has v0, but is called Best (lol) https://github.com/DoumanAsh/mp3lame-encoder

// metadata
// mp3/wav/aiff https://github.com/polyfloyd/rust-id3
// mp3/m4a/flac https://github.com/TianyiShi2001/audiotags -- i don't like the typing (Box? Album?)
// TODO: opus

/// Mainly for transcoding. For metadata, id3 is always used.
#[derive(Debug)]
pub enum FileType {
    // Lossy
    MP3,
    OPUS,

    // Lossless
    WAV,
    /// Handled by claxon
    FLAC,

    Unknown,
}

impl Walk for DirEntry {
    fn walk(&self) -> impl Iterator<Item = DirEntry> {
        WalkDir::new(self.path())
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
    }
    /// UTF-8 errors are ignored
    fn as_str(&self) -> &str { self.path().to_str().unwrap() }
    fn as_dir(&self) -> DirEntry { self.clone() }
    /// basename
    fn as_list_item(&self) -> ListItem {
        // ListItem::new(self.as_str()) // fullpath
        // TODO: color based on filetype
        ListItem::new(self.path().file_name().unwrap().to_str().unwrap())
    }
}

/// Wrapper over `id3::Tag`. It is important to note that metadata can be read
/// and stored completely separately from the audio file. Implements some
/// transcoding methods for convenience.
#[derive(Debug)]
pub struct File {
    pub path: String,

    pub file_type: FileType,

    /// To avoid the need for an adapter over different audio formats and tag
    /// containers (and since I always transcode into MP3), we default to
    /// `id3`. Under the hood, other crates like `lofty` are used.
    pub tags: id3::Tag,
}

/// Used in `Track` and `File`
#[derive(Debug)]
pub enum TagField {
    Artist,
    Album,
    Year,
    Title,
    TrackNumber,
    Genre,
}

impl File {
    pub fn new(path: &str) -> anyhow::Result<Self> {
        let ft = infer::get_from_path(path)
            .context("read file")?
            .context("infer filetype")?
            .extension(); // note: this disregards the actual filetype

        let file_type = match ft {
            "mp3" => FileType::MP3,
            "flac" => FileType::FLAC,
            _ => FileType::Unknown,
        };

        // init with empty tags, so we can use File.get for convenience
        let f = Self {
            path: path.to_string(),
            file_type,
            tags: {
                match id3::Tag::read_from_path(path) {
                    Ok(tags) => tags,
                    _ => id3::Tag::new(),
                }
            },
        };

        Ok(f)
    }

    fn copy_flac_tags(
        &mut self,
        new_path: &str,
    ) -> Result<()> {
        // metaflac: vorbis comments are stored internally as hashmap, but API doesn't
        // let you get them in any way --
        // https://jameshurst.github.io/rust-metaflac/metaflac/block/struct.VorbisComment.html

        // // symphonia_metadata requires a MetadataBuilder (whatever that is)
        // symphonia_metadata::flac::read_comment_block(reader, metadata);
        // symphonia_metadata::id3v2::read_id3v2(reader, metadata);

        // lofty is probably the cleanest way to do it
        let mut buf = std::fs::File::open(&self.path)?;
        let flacfile = lofty::flac::FlacFile::read_from(&mut buf, ParseOptions::default())?;
        let comments = flacfile.vorbis_comments().context("no vorbis comments")?;

        for (tag, com) in [
            // 2nd value should be [&str], probably
            (TagField::Title, "TITLE"),
            (TagField::TrackNumber, "TRACKNUMBER"),
            (TagField::Artist, "ARTIST"),
            (TagField::Album, "ALBUM"),
            (TagField::Year, "DATE"),
        ] {
            if let Some(val) = comments.get(com) {
                self.set(tag, val);
            }
        }

        self.tags.write_to_path(new_path, id3::Version::Id3v24)?;

        Ok(())
    }

    pub fn get(
        &self,
        field: TagField,
    ) -> Option<String> {
        match field {
            // i hate &str so much
            TagField::Artist => self.tags.artist().map(|f| f.to_string()),
            TagField::Album => self.tags.album().map(|f| f.to_string()),
            TagField::Title => self.tags.title().map(|f| f.to_string()),
            TagField::Year => self.tags.year().map(|f| f.to_string()),
            TagField::TrackNumber => self.tags.track().map(|f| f.to_string()),
            TagField::Genre => self.tags.genre_parsed().map(|f| f.to_string()),
            // _ => None,
        }
        // .map(|f| f.to_string())
    }

    fn set(
        &mut self,
        field: TagField,
        value: &str,
    ) {
        // Tag.set_X cannot fail, apparently
        match field {
            TagField::Title => self.tags.set_title(value),
            TagField::Artist => self.tags.set_artist(value),
            TagField::Album => self.tags.set_album(value),
            TagField::Year => self.tags.set_year(value.parse().unwrap()),
            TagField::TrackNumber => self.tags.set_track(value.parse().unwrap()),
            TagField::Genre => self.tags.set_genre(value),
        }
    }

    /// The target encoding is always MP3 V0 (for now). Shell commands are used
    /// because I haven't found a crate that does lossy transcoding at a low
    /// level.
    ///
    /// - Extract tags as id3 (if present)
    /// - Transcode to mp3
    /// - Write id3 tags to new mp3 file
    pub fn transcode(&mut self) -> Result<()> {
        // TODO: check bitrate
        if let FileType::MP3 = self.file_type {
            // let buf = std::fs::File::open(&self.path)?;

            // reading entire streams is impractical

            // let f = puremp3::Mp3Decoder::new(buf)
            //     .frames()
            //     .last()
            //     .unwrap()
            //     .num_samples;
            // println!("{}", f); // 20 s / 2.3 MB

            // let (_, stream) = puremp3::read_mp3(buf).unwrap();
            // println!("{}", stream.count()); // 20 s / 2.3 MB

            // panic!();
            return Ok(());
        }

        // "lame --silent -V 0 --disptime 1";

        // https://doc.rust-lang.org/std/process/index.html#handling-io

        // "flac in.flac --decode --stdout --totally-silent |
        // lame --silent -V 0 - out.mp3"

        // println!("{:#?}", f.tags);

        let outfile = format!("{}.mp3", self.path);

        match self.file_type {
            FileType::FLAC => {
                let flac = Command::new("flac")
                    .arg(&self.path)
                    .args("--decode --stdout --totally-silent".split_whitespace())
                    .stdout(Stdio::piped())
                    .spawn()?;
                let mut lame = Command::new("lame")
                    .args("-V 0 -".split_whitespace())
                    // if you decide to collect the output bytes and write the buffer yourself,
                    // the new file will have incorrect duration
                    .arg(&outfile)
                    .stdin(Stdio::from(flac.stdout.context("no flac stdout")?))
                    .spawn()?;
                lame.wait()?;
            }

            // "ffmpeg -y -i";
            // "lame --silent {BITRATE_ARG} --disptime 1";
            _ => unimplemented!(),
        };

        self.copy_flac_tags(&outfile)?;
        // TODO: remove self.path
        self.path = outfile;

        Ok(())
    }
}

impl Display for File {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        writeln!(f, "{}", self.path)?;
        writeln!(f, "title: {}", self.tags.title().unwrap_or("none"))?;
        writeln!(f, "artist: {}", self.tags.artist().unwrap_or("none"))?;
        writeln!(f, "album: {}", self.tags.album().unwrap_or("none"))?;
        writeln!(f, "year: {}", self.tags.year().unwrap_or(0))?;
        Ok(())
    }
}

pub struct SourceDir {
    pub path: String,
    pub dir: DirEntry,
}
impl SourceDir {
    pub fn new(path: &str) -> Result<Self> {
        let dir = WalkDir::new(path)
            .into_iter()
            .next()
            .expect("root dir returned")?;
        Ok(Self {
            path: path.to_string(),
            dir,
        })
    }

    pub fn tags(&self) -> Vec<File> {
        self.dir
            .sort(true)
            .iter()
            .map(|f| f.as_str())
            .map(File::new)
            .filter_map(|p| p.ok())
            .collect()
    }

    pub fn dirs(&self) -> Vec<DirEntry> { self.dir.sort(false) }
}

// should probably be used as the return type for matches_discogs (instead of
// bool), so that we can decide how to handle parse errors
pub enum ParseError {
    /// Generally unrecoverable
    UnequalLen,

    /// Can usually be ignored
    UnequalDur,
    BadTags,
}

impl SourceDir {
    /// Some quirks:
    ///
    /// - `TagLike::duration()` may return `Some(0)`, for some reason; not sure
    ///   if `None` can be returned
    /// - durations are returned in milliseconds, so we convert to seconds
    pub fn durations(&self) -> Vec<Option<u32>> {
        self.tags()
            .iter()
            // warning: duration may be inaccurate if not properly encoded
            .map(|t| t.tags.duration()) //.unwrap_or(0))
            .map(|d| d.map(|d| d / 1000)) // Option.map in Iterator.map is wild
            .collect()
    }

    pub fn matches_discogs(
        &self,
        rel: &Release,
    ) -> bool {
        if self.tags().len() != rel.tracklist().len() {
            return false;
        }

        let diffs = zip(self.durations(), rel.durations()).map(|(a, b)| a.unwrap_or(0).abs_diff(b));
        if diffs.max() > Some(5) {
            return false;
        }

        true
    }

    pub fn apply_discogs(
        &mut self,
        rel: &Release,
    ) -> Result<()> {
        for (discogs_track, file) in rel.tracklist().iter().zip(&mut self.tags()) {
            // println!("{}\n{}", discogs_track, file);

            // println!("{}\n{:?}", discogs_track.title, file.get(TagField::Title));

            // let dist = levenshtein(&discogs_track.title,
            // &file.get(TagField::Title).unwrap()); println!("{}", dist);

            // println!("{}\n{:?}", rel.title, file.get(TagField::Album));
            // println!("{}\n{:?}", rel.artists_sort, file.get(TagField::Artist));
            // println!("{}\n{:?}", rel.year, file.get(TagField::Year));

            file.set(TagField::Title, &discogs_track.title);
            file.set(TagField::Artist, &rel.artists_sort);
            file.set(TagField::Album, &rel.title);
            file.set(TagField::Year, &rel.year.to_string());
            file.tags.write_to_path(&file.path, id3::Version::Id3v24)?;

            //
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    //{{{

    use lofty::AudioFile;
    use lofty::ParseOptions;

    use crate::transcode::File;

    fn test_duration() {
        let infile = "foo.flac";
        let outfile = "foo.flac.mp3";

        File::new(infile).unwrap().transcode().unwrap();

        let mut buf = std::fs::File::open(infile).unwrap();
        let flacfile = lofty::flac::FlacFile::read_from(&mut buf, ParseOptions::default()).unwrap();
        let flacdur = flacfile.properties().duration().as_secs();

        let mut buf = std::fs::File::open(outfile).unwrap();
        let newfile = lofty::mpeg::MpegFile::read_from(&mut buf, ParseOptions::default()).unwrap();
        let mp3dur = newfile.properties().duration().as_secs();

        assert_eq!(flacdur, mp3dur);
    }
} //}}}
