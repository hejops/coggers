//! File operations, chiefly:
//!
//! 1. Directories in the library (<> SQL)
//! 2. Directories to be tagged (-> ID3, etc)

use std::env;
use std::fs;
use std::path::Path;

use anyhow::anyhow;
use lazy_static::lazy_static;
use ratatui::widgets::ListItem;
/* see also: https://github.com/matklad/once_cell, https://blog.logrocket.com/rust-lazy-static-pattern/#differences-between-lazy-static-oncecell-lazylock */
use rusqlite::Connection;
use rusqlite::Result;
use walkdir::DirEntry;
use walkdir::WalkDir;

use crate::transcode::File;

// hyperfine 'find $MU >/dev/null'
//   Time (mean ± σ):      1.123 s ±  0.003 s

// hyperfine 'cargo run >/dev/null'
//   Time (mean ± σ):      1.631 s ±  0.219 s

lazy_static! {
    // TODO: load from config file ~/.config/coggers/config
    pub static ref LIBRARY_ROOT: String = env::var("MU").expect("Environment variable $MU must be set");
    pub static ref SOURCE: String = env::var("SOURCE").expect("Environment variable $SOURCE must be set");
}

/// Helper trait to simplify interconversion between String/&str and DirEntry.
pub trait Walk {
    /// Return `Iterator`, collection is deferred to callers.
    fn walk(&self) -> impl Iterator<Item = DirEntry>;
    fn as_str(&self) -> &str;
    fn as_dir(&self) -> DirEntry;
    fn as_list_item(&self) -> ListItem;
}

impl Walk for &str {
    fn walk(&self) -> impl Iterator<Item = DirEntry> {
        WalkDir::new(self).into_iter().filter_map(|f| f.ok())
    }
    fn as_str(&self) -> &str { self }
    fn as_dir(&self) -> DirEntry { WalkDir::new(self).into_iter().next().unwrap().unwrap() }
    fn as_list_item(&self) -> ListItem { ListItem::new(self.as_str()) }
}

pub trait Sort {
    fn sort(
        &self,
        files_only: bool,
    ) -> Vec<DirEntry>;
}

impl Sort for DirEntry {
    /// Walk 1 level, collect files, return as sorted vec
    fn sort(
        &self,
        files: bool,
    ) -> Vec<DirEntry> {
        let x = WalkDir::new(self.as_str())
            .min_depth(1)
            .max_depth(1)
            .into_iter()
            // .filter_entry is more strict than filter, as iteration is stopped as soon as the
            // first predicate is false; in this case, the first item is the dir itself,
            // which returns false!
            .filter_map(|e| e.ok());

        let mut files: Vec<DirEntry> = match files {
            true => x.filter(|e| e.file_type().is_file()).collect(),
            false => x.filter(|e| e.file_type().is_dir()).collect(),
        };

        files.sort_by(|a, b| a.as_str().cmp(b.as_str()));
        files
    }
}

impl File {
    pub fn new(f: &DirEntry) -> anyhow::Result<Self> {
        assert!(f.file_type().is_file());
        let path = f.as_str();
        let tag = id3::Tag::read_from_path(path)?;
        Ok(Self {
            path: path.to_string(),
            tags: tag,
        })
    }
}

pub struct Library {
    root: String,
    dirs: Vec<DirEntry>,
}

impl Library {
    /// Initialise a possibly empty `Library`; the directories are NOT collected
    /// automatically, primarily for testing purposes. This makes it rather
    /// awkward for callers:
    ///
    /// ```rust
    /// use discogs::io::Library;
    /// use discogs::io::Walk;
    /// use discogs::io::LIBRARY_ROOT;
    ///
    /// let lib = Library::new(&LIBRARY_ROOT);
    /// let dirs = lib.walk(); // as Iterator
    ///
    /// // let lib = Library::new(&LIBRARY_ROOT).collect(); // as Vec, slow
    /// ```
    pub fn new(root: &str) -> Self {
        let root = root.to_string();
        let dirs = vec![];
        Self { root, dirs }
    }
    pub fn collect(mut self) -> Self {
        self.dirs = self.walk().collect();
        self
    }
}

impl Walk for Library {
    /// Directories only, of the form `<root>/<artist>/<album>`. We are largely
    /// unconcerned with the files contained within, as they are meant to be
    /// played.
    fn walk(&self) -> impl Iterator<Item = DirEntry> {
        // fn walk(root: &str) -> impl Iterator<Item = DirEntry> {
        WalkDir::new(&self.root)
            .min_depth(2)
            .max_depth(2)
            .into_iter()
            .filter_entry(|e| e.file_type().is_dir())
            .filter_map(|e| e.ok())
        // returning a Filter[Map] at compile time is relatively easy, returning
        // a Map is not -- https://stackoverflow.com/a/27497032
    }
    fn as_dir(&self) -> DirEntry {
        WalkDir::new(&self.root)
            .into_iter()
            .next()
            .unwrap()
            .unwrap()
    }
    fn as_str(&self) -> &str { &self.root }
    fn as_list_item(&self) -> ListItem { ListItem::new(self.as_str()) }
}

#[derive(Debug, PartialEq)]
/// Data structure shared between Library and Database
pub struct LibraryEntry {
    pub artist: String,
    pub album: String,
    pub year: usize,
}
impl LibraryEntry {
    /// Parse path in the form 'artist/album (year)'
    pub fn from_path(path: DirEntry) -> anyhow::Result<Self> {
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

/// SQL representation of Library (potentially very confusing, so maybe should
/// be merged into Library?)
///
/// The directory structure is strictly adhered to:
///     `<db_path>/<artist>/<album> (<year>)`
pub struct LibraryDB {
    db_path: String,
    pub entries: Vec<LibraryEntry>,
}

impl LibraryDB {
    /// Load from static sqlite db.
    pub fn load(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        let mut stmt = conn.prepare("select * from albums;")?;
        let entries: Vec<LibraryEntry> = stmt
            .query_map([], |row| {
                Ok(LibraryEntry {
                    artist: row.get(0)?,
                    album: row.get(1)?,
                    year: row.get(2)?,
                })
            })?
            .filter_map(|e| e.ok())
            .collect();
        Ok(Self {
            db_path: db_path.to_string(),
            entries,
        })
    }

    /// Traverse the music directory and dump all results in a sqlite3 database.
    ///
    /// 9 min / 4 TB / 59 k albums (cold)
    /// 2 min / 4 TB / 59 k albums (warm)
    pub fn dump(&self) -> rusqlite::Result<()> {
        if Path::new(&self.db_path).exists() {
            fs::remove_file(&self.db_path).unwrap();
        };
        let conn = Connection::open(&self.db_path)?;

        conn.execute(
            "create table if not exists albums (
             artist text not null,
             album text not null,
             year integer not null
         )",
            [],
        )?;

        for a in Library::new(&LIBRARY_ROOT)
            .collect()
            .dirs
            .into_iter()
            .map(LibraryEntry::from_path)
            .filter_map(|a| a.ok())
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
}

#[cfg(test)]
mod tests {
    use crate::io::Library;
    use crate::io::LibraryDB;
    use crate::io::LibraryEntry;
    use crate::io::Walk;
    use crate::io::LIBRARY_ROOT;

    #[test]
    fn test_album_dir() {
        let first_dir = Library::new(&LIBRARY_ROOT).walk().next().unwrap();
        assert!(LibraryEntry::from_path(first_dir.clone()).is_ok());
    }

    #[test]
    fn test_db_load() {
        let db = LibraryDB::load("test.db").unwrap();
        let first = db.entries.first();
        assert!(first.is_some());
        assert!(db.entries.len() > 1);
    }
}
