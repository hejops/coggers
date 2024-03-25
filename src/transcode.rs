//! Transcoding and preservation of metadata across formats

use walkdir::DirEntry;
use walkdir::WalkDir;

use crate::io::Walk;

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
// mp3 (id3) https://github.com/polyfloyd/rust-id3
// mp3/m4a/flac https://github.com/TianyiShi2001/audiotags
// TODO: opus

pub struct File {
    path: String,
    file_type: FileType,
}

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

// i initially wanted to write a method dir.parse, but that violates the orphan
// rule (https://doc.rust-lang.org/error_codes/E0210.html), so i just made a trait lol
pub trait Parse {
    fn parse(&self);
}

impl Parse for DirEntry {
    // TODO: figure out the appropriate scope for this fn; most likely, just need to
    // return Vec<Tag>
    fn parse(&self) {
        let files: Vec<DirEntry> = WalkDir::new(self.path())
            .into_iter()
            // .filter_entry is more strict than filter, as iteration is stopped as soon as the
            // first predicate is false; in this case, the first item is the dir itself,
            // which returns false!
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .collect();
        let file = files.first().unwrap();
        // let tags = audiotags::FlacTag::read_from_path(file.path());
        let tags = audiotags::Tag::new().read_from_path(file.path()); // equivalent
        println!("{:?}", file);
        println!("{:#?}", tags.unwrap().album());
    }
}
