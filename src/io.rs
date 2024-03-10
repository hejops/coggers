use std::env;
use std::fs::remove_file;
use std::path::Path;

use anyhow::anyhow;
use anyhow::Result;
use lazy_static::lazy_static; // see also: https://github.com/matklad/once_cell
use rusqlite::Connection;
use walkdir::DirEntry;
use walkdir::WalkDir;

// hyperfine 'find $MU >/dev/null'
//   Time (mean ± σ):      1.123 s ±  0.003 s

// hyperfine 'cargo run >/dev/null'
//   Time (mean ± σ):      1.631 s ±  0.219 s

lazy_static! {
    // TODO: load from config file
    static ref LIBRARY_ROOT: String = env::var("MU").expect("Environment variable $MU must be set");
}

pub fn walk() -> impl Iterator<Item = DirEntry> {
    WalkDir::new(LIBRARY_ROOT.as_str())
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
        let path = path.path().strip_prefix(LIBRARY_ROOT.as_str())?;
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

/// Traverse the music directory and dump all results in a sqlite3 database.
///
/// 10.5 min / 4 TB / 59 k albums (cold?, late depth filter)
/// 2 min / 4 TB / 59 k albums (warm?, early depth filter)
pub fn dump_db() -> rusqlite::Result<()> {
    if Path::new("test.db").exists() {
        remove_file("test.db").unwrap();
    };
    let conn = Connection::open("test.db")?;

    conn.execute(
        "create table if not exists albums (
             artist text not null,
             album text not null,
             year integer not null
         )",
        [],
    )?;

    for a in walk().map(AlbumDir::from_path).filter_map(|a| a.ok())
    // note to self: trying to be lazy all the way means the side effects do not get executed!
    // .map(|a| { conn.execute(...) } )
    {
        conn.execute(
            "INSERT INTO albums (artist, album, year) values (?1, ?2, ?3)",
            [a.artist, a.album, a.year.to_string()],
        )?;
    }

    Ok(())
}

// let mut stmt = conn.prepare("select * from albums;")?;
// let entries: Vec<AlbumDir> = stmt
//     .query_map([], |row| {
//         Ok(AlbumDir {
//             artist: row.get(0)?,
//             album: row.get(1)?,
//             year: row.get(2)?,
//         })
//     })?
//     .filter_map(|e| e.ok())
//     .collect();
// println!("{:#?}", entries);
