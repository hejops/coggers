//! Transcoding and preservation of metadata across formats

use std::fmt::Display;
use std::iter::zip;

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

/// Mainly for transcoding (symphonia?). For metadata, id3 is always used.
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
    // do errors need to be handled?
    fn as_str(&self) -> &str { self.path().to_str().unwrap() }
    fn as_dir(&self) -> DirEntry { self.clone() }
    fn as_list_item(&self) -> ListItem {
        //
        // ListItem::new(self.as_str()) // fullpath
        // TODO: color based on filetype
        ListItem::new(self.path().file_name().unwrap().to_str().unwrap())
    }
}

/// Wrapper over `id3::Tag`
#[derive(Debug)]
pub struct File {
    pub path: String,
    pub file_type: FileType,
    /// To avoid the need for an adapter over different tag containers (and
    /// since I always transcode into MP3), id3 is used.
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
}

impl File {
    pub fn new(path: &str) -> anyhow::Result<Self> {
        let ft = infer::get_from_path(path)
            .expect("read file")
            .expect("infer filetype")
            .extension(); // disregards the actual filetype

        let ft = match ft {
            "mp3" => FileType::MP3,
            "flac" => FileType::FLAC,
            _ => FileType::Unknown,
        };

        // init with empty tags, so we can use File.get for convenience
        let mut f = Self {
            path: path.to_string(),
            file_type: ft,
            tags: id3::Tag::new(),
        };

        match id3::Tag::read_from_path(path) {
            Ok(tags) => f.tags = tags,
            Err(_) => {
                // parse flac tags and cast them into id3; after conversion into mp3, they can
                // be set

                // metaflac: vorbis comments are stored internally as hashmap, but API doesn't
                // let you get them in any way --
                // https://jameshurst.github.io/rust-metaflac/metaflac/block/struct.VorbisComment.html

                // // symphonia_metadata requires a MetadataBuilder (whatever that is)
                // symphonia_metadata::flac::read_comment_block(reader, metadata);
                // symphonia_metadata::id3v2::read_id3v2(reader, metadata);

                // lofty is probably the cleanest way to do it

                let mut buf = std::fs::File::open(path).unwrap();
                let flacfile = lofty::flac::FlacFile::read_from(&mut buf, ParseOptions::default())?;
                let comments = flacfile.vorbis_comments().unwrap();

                println!("{:?}", comments);

                for (tag, com) in [
                    // 2nd value should be [&str], probably
                    (TagField::Title, "TITLE"),
                    (TagField::TrackNumber, "TRACKNUMBER"),
                    (TagField::Artist, "ARTIST"),
                    (TagField::Album, "ALBUM"),
                    (TagField::Year, "DATE"),
                ] {
                    if let Some(val) = comments.get(com) {
                        f.set(tag, val);
                    }
                }

                println!("{:?}", f.tags);
                // nothing to save yet
            }
        };

        Ok(f)
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
        }
    }

    pub fn transcode() {}
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
    // i refuse to allow a `state: ListState` field, as the data and interface must be kept
    // separate
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
