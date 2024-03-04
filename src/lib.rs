pub mod http; // re-export for main, and to silence dead_code
pub mod release;

#[cfg(test)]
mod tests {
    use crate::release::get_release;
    use crate::release::Release;

    #[test]
    fn test_release() {
        // https://www.discogs.com/release/8196883
        let rel: Release = get_release(8196883).unwrap();
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
    }

    #[test]
    fn test_nonexistent_release() {
        assert_eq!(get_release(0), None);
    }
}
