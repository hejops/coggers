use std::env;

use anyhow::anyhow;
use anyhow::Result;
use walkdir::DirEntry;
use walkdir::WalkDir;

// hyperfine 'find $MU >/dev/null'
//   Time (mean ± σ):      1.123 s ±  0.003 s

// hyperfine 'cargo run >/dev/null'
//   Time (mean ± σ):      1.631 s ±  0.219 s

pub fn walk() -> impl Iterator<Item = DirEntry> {
    let d = env::var("MU").expect("env var"); // TODO: lazy_static

    WalkDir::new(d)
        .min_depth(2)
        .max_depth(2)
        .into_iter()
        .filter_entry(|e| e.file_type().is_dir())
        .filter_map(|e| e.ok())
    // returning a Filter at compile time is relatively easy, returning a Map is
    // not -- https://stackoverflow.com/a/27497032
    // .map(|entry| entry.path().strip_prefix(d).unwrap())
}

#[derive(Debug)]
pub struct AlbumDir {
    artist: String,
    album: String,
    year: usize,
}
impl AlbumDir {
    /// Parse path in the form 'artist/album (year)'
    pub fn from_path(path: DirEntry) -> Result<Self> {
        let d = env::var("MU").expect("env var");
        let path = path.path().strip_prefix(d)?;
        let mut path_iter = path
            .to_str()
            .ok_or_else(|| anyhow!("could not convert path to string"))?
            .split('/');
        let artist = path_iter.next().unwrap().to_string(); // first iter should always succeed
        let (album, year) = path_iter
            .next()
            .ok_or_else(|| anyhow!("no / found"))?
            .rsplit_once('(')
            .ok_or_else(|| anyhow!("no ( found"))?;
        let album = album.trim().to_string();
        let year = year
            .strip_suffix(')')
            .ok_or_else(|| anyhow!("no trailing )"))?
            .parse()?;
        Ok(Self {
            artist,
            album,
            year,
        })
    }
}
