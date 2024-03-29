//! Transcoding and preservation of metadata across formats

use std::fmt::Display;
use std::iter::zip;

use anyhow::Result;
use id3::TagLike;
use walkdir::DirEntry;
use walkdir::WalkDir;

use crate::io::get_sorted_files;
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

///
pub enum FileType {
    // Lossy
    MP3,
    OPUS,

    // Lossless
    WAV,
    /// Handled by claxon
    FLAC,
}

/// Assumed to be a flat listing of directories only (?), of the form
/// `<source>/<dir>`.
pub struct Source {
    root: String,
    pub dirs: Vec<DirEntry>,
}

impl Source {
    /// This is literally identical to `Library::new`; there must be a better
    /// way to do this...
    //
    // the root cause of this duplication is that we want to generalise Walk to
    // WalkDir::DirEntry, which does not need to have a new(). thus, for our own
    // structs, we need to first init a self with empty .dirs, just to be able
    // to call .walk()
    pub fn new(root: &str) -> Self {
        let root = root.to_string();
        let dirs = vec![];
        let mut lib = Self { root, dirs };
        lib.dirs = lib.walk().collect();
        lib
    }
}

impl Walk for Source {
    fn walk(&self) -> impl Iterator<Item = DirEntry> {
        WalkDir::new(&self.root)
            .min_depth(1)
            .max_depth(1)
            .into_iter()
            .filter_entry(|e| e.file_type().is_dir())
            .filter_map(|e| e.ok())
    }
}

impl Walk for DirEntry {
    fn walk(&self) -> impl Iterator<Item = DirEntry> {
        WalkDir::new(self.path())
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
    }
}

/// Wrapper over `audiotags::Tag`
pub struct File {
    pub path: String,
    // raw_tags: Box<dyn AudioTag + Send + Sync>, // audiotags
    pub tags: id3::Tag,
}

/// Used in `Track` and `File`
pub enum TagField {
    Artist,
    Album,
    Year,
    Title,
}

impl File {
    fn get(
        &self,
        field: TagField,
    ) -> Option<String> {
        match field {
            // i hate &str so much
            TagField::Artist => self.tags.artist().map(|f| f.to_string()),
            TagField::Album => self.tags.album().map(|f| f.to_string()),
            TagField::Title => self.tags.title().map(|f| f.to_string()),
            TagField::Year => self.tags.year().map(|f| f.to_string()),
            // _ => None,
        }
        // .map(|f| f.to_string())
    }

    fn set(
        &mut self,
        field: TagField,
        value: &str,
    ) {
        // set_X cannot fail, apparently
        match field {
            TagField::Title => self.tags.set_title(value),
            TagField::Artist => self.tags.set_artist(value),
            TagField::Album => self.tags.set_album(value),
            TagField::Year => self.tags.set_year(value.parse().unwrap()),
        }
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
    path: String,
    pub tags: Vec<File>,
}
impl SourceDir {
    pub fn new(path: &str) -> Result<Self> {
        let dir = WalkDir::new(path)
            .into_iter()
            .next()
            .expect("root dir returned")?;
        let files = get_sorted_files(&dir)
            .iter()
            // .map(|path| File::new(dir_to_str(path)))
            .map(File::new)
            .filter_map(|p| p.ok())
            .collect();
        Ok(Self {
            path: path.to_string(),
            tags: files,
        })
    }
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
        self.tags
            .iter()
            .map(|t| t.tags.duration()) //.unwrap_or(0))
            .map(|d| d.map(|d| d / 1000)) // Option.map in Iterator.map is wild
            .collect()
    }

    pub fn matches_discogs(
        &self,
        rel: &Release,
    ) -> bool {
        if self.tags.len() != rel.tracklist().len() {
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
        for (discogs_track, file) in rel.tracklist().iter().zip(&mut self.tags) {
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
