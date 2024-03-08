pub mod http; // re-export for main, and to silence dead_code
pub mod io;
pub mod release;
pub mod search;

#[cfg(test)]
mod tests {
    use crate::io::walk;
    use crate::io::AlbumDir;
    use crate::release::Release;
    use crate::search::search_release;

    #[test]
    fn test_release() {
        // https://www.discogs.com/release/8196883
        let rel = Release::get(8196883).unwrap();
        assert_eq!(rel.year, 1998);
        assert_eq!(rel.uri, "https://www.discogs.com/release/8196883-Monica-Groop-Ostrobothnian-Chamber-Orchestra-Conductor-Juha-Kangas-Bach-Alto-Cantatas");
        assert_eq!(rel.id, 8196883);
        assert_eq!(rel.genres, vec!["Classical"]);
        assert_eq!(rel.artists[0].name, "Monica Groop");

        assert_eq!(
            rel.tracklist[0].title,
            "J.S.Bach: Cantatas BWV 170 / 35 / 169"
        );
        assert_eq!(
            rel.tracklist[1].sub_tracks.as_ref().unwrap()[0].title,
            "Aria \"Vergnügte Ruh, Beliebte Seelenlust\"",
        );

        assert_eq!(rel.parse_tracklist().len(), 19);
    }

    #[test]
    fn test_nonexistent_release() {
        assert_eq!(Release::get(0), None);
    }

    #[test]
    fn test_big_search() {
        let album = "ride the lightning";
        let artist = "metallica";
        assert_eq!(search_release(artist, album).unwrap().len(), 50);
    }

    #[test]
    fn test_empty_search() {
        let album = "djsakldjsakl";
        let artist = "metallica";
        assert_eq!(search_release(artist, album), None);
    }

    #[test]
    fn test_album_dir() {
        assert!(AlbumDir::from_path(walk().next().unwrap()).is_ok());
    }
}
