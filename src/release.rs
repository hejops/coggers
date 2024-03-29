use std::fmt::Display;

use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;

use crate::http;
use crate::search::SearchResults;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
enum TrackType {
    // TODO: rename_all
    #[serde(rename = "index")]
    Index,
    #[serde(rename = "track")]
    Track,
    #[serde(rename = "heading")]
    Heading,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Track {
    pub title: String,
    /// May be an empty string (not None)
    pub duration: String,
    type_: TrackType,
    pub sub_tracks: Option<Vec<Track>>,
}

impl Display for Track {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        // no newlines should be inserted here
        if !self.duration.is_empty() {
            write!(f, "[{}] ", self.duration)?;
        }
        write!(f, "{}", self.title)?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Label {
    pub id: usize,
    pub name: String,
    // other fields not implemented yet
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Artist {
    // TODO: empty strings should be deserialised to None
    // https://docs.rs/serde_with/latest/serde_with/struct.NoneAsEmptyString.html
    pub anv: String,
    pub id: usize,
    pub name: String,
    pub resource_url: String,
    pub role: String,   // is this an enum? i doubt it
    pub tracks: String, // this being a String is very problematic
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Master {
    /// AKA primary
    pub main_release: usize,
    main_release_url: String,

    most_recent_release: usize,
    most_recent_release_url: String,

    pub id: usize, // u32 is probably fine
    pub year: u16,

    pub data_quality: String, // TODO: enum
    /// API endpoint, in the form api.discogs.com/...
    pub resource_url: String,
    /// URL in the form https://www.discogs.com/release/123-title
    pub uri: String,

    /// Genres are distinct from styles.
    pub genres: Vec<String>,
    pub notes: String,
    pub title: String,

    pub artists: Vec<Artist>,
    /// Should correspond to the tracklist of the primary release (need to find
    /// counterexamples)
    pub tracklist: Vec<Track>,

    lowest_price: f32,
    num_for_sale: usize,
}

/// The definitive representation of a release, and the only one with tracklist.
/// Similar to CollectionRelease and SearchRelease, both of which contain less
/// information.
///
/// Discogs does not provide a way to tell whether a release is primary; only
/// master releases have this information.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Release {
    // Explicitly defining multiple structs allows us to know all available fields at compile time,
    // providing a more ergonomic experience for callers (less Option checking), at the expense of
    // a fair amount of struct duplication. An alternative to consider would be to define a single
    // struct with all fields, but leave some as Option<T>s.
    /// https://support.discogs.com/hc/en-us/articles/4402686008589-Why-Are-Some-Items-Blocked-In-The-Discogs-Marketplace
    pub blocked_from_sale: bool,

    /// Unique identifier of a Release.
    pub id: usize, // u32 is probably fine
    /// The earliest possible year is somewhere in the 1920s.
    pub year: u16,

    pub data_quality: String, // TODO: enum
    pub status: String,       // TODO: enum

    /// API endpoint, in the form api.discogs.com/...
    pub resource_url: String,
    /// URL in the form https://www.discogs.com/release/123-title
    pub uri: String,

    /// Single string representation of the (main) Artists involved in a
    /// Release.
    pub artists_sort: String,
    pub country: String,

    /// Genres are distinct from styles.
    pub genres: Vec<String>,
    pub notes: String,
    pub title: String,

    pub artists: Vec<Artist>,
    pub extraartists: Vec<Artist>,
    pub labels: Vec<Label>,
    /// Raw representation of tracklist. This field is private, as its use is
    /// discouraged; `Release.tracklist()`, which better handles potential
    /// nesting, should be used instead.
    tracklist: Vec<Track>,

    // companies: Vec,
    // formats: Vec,
    // identifiers: Vec,
    /// The currency is assumed from the locale and not specified in the
    /// response.
    lowest_price: f32,
    num_for_sale: usize,
}

impl Release {
    /// Used when the release ID is known; otherwise, use Release::search.
    /// Note: passing a master ID will produce incorrect release! Use
    /// Release::get_master instead.
    ///
    /// Returns None if release is not found.
    pub fn get(release_id: usize) -> Option<Self> {
        let resp = http::make_request(http::RequestType::Release, &release_id.to_string()).ok()?;

        match resp.status() {
            reqwest::StatusCode::OK => serde_json::from_str(resp.text().unwrap().as_str()).unwrap(),
            _ => None,
        }
    }

    // this should probably be Master::get?
    // having a bunch of get functions is becoming annoying, maybe time to make a
    // trait Get?
    pub fn get_master(master_id: usize) -> Option<Master> {
        let resp = http::make_request(http::RequestType::Master, &master_id.to_string()).ok()?;

        match resp.status() {
            reqwest::StatusCode::OK => serde_json::from_str(resp.text().unwrap().as_str()).unwrap(),
            _ => None,
        }
    }

    /// 50 per page. Returns None if no search results. Filtering is not handled
    /// here.
    pub fn search(
        artist: &str,
        album: &str,
        // ) -> Option<Vec<SearchRelease>> {
    ) -> SearchResults {
        // None is used over empty Vec as it better signals intent
        let resp = http::make_request(
            http::RequestType::Search,
            &format!("/database/search?release_title={album}&artist={artist}&type=release"),
        )
        .unwrap();
        let results: SearchResults = serde_json::from_str(resp.text().unwrap().as_str()).unwrap();
        results
        // cast empty vec into None -- https://stackoverflow.com/a/65012849
        // (!results.results.is_empty()).then_some(results.results)
    }

    pub fn durations(&self) -> Vec<u32> {
        fn as_int(dur: &str) -> Result<u32> {
            let mut int: u32 = 0;
            for (i, part) in dur.split(':').rev().enumerate() {
                int += part.parse::<u32>()? * 60_u32.pow(i.try_into()?);
            }
            Ok(int)
        }
        self.tracklist()
            .iter()
            .map(|t| t.duration.as_ref())
            .map(|dur| as_int(dur).unwrap_or(0))
            .collect()
    }

    /// Extract Discogs tracklist (which may be nested) as a flat list.
    pub fn tracklist(&self) -> Vec<&Track> {
        fn recurse(tracks: &[Track]) -> Vec<&Track> {
            let mut out = vec![];
            for track in tracks.iter() {
                match &track.sub_tracks {
                    Some(sub) => out.append(&mut recurse(sub)),
                    None => {
                        if track.type_ == TrackType::Track {
                            out.push(track);
                        }
                    }
                }
            }
            out
        }
        recurse(&self.tracklist)
    }

    /// Precedence: track credits > album credits > artists_sort.
    ///
    /// This is strictly for classical releases; outside classical music, it is
    /// usually more meaningful to use artists_sort.
    ///
    /// The length of the returned Vec will either be 1 or equal to the length
    /// of the parsed tracklist.
    pub fn get_composers(&self) -> Option<Vec<&str>> {
        if !self.genres.iter().any(|g| g == "Classical") {
            return None;
        }

        // TODO: track credits

        let extra: Vec<&str> = self
            .extraartists
            .iter()
            .filter(|a| a.role.starts_with("Compose"))
            .map(|a| a.name.as_str())
            .collect();
        Some(match extra.len() {
            1 => extra,
            _ => vec![&self.artists_sort],
        })
    }
}

// TODO: group Display impls in display.rs?
/// Tracklist will not be included; use display_tracklist instead
impl Display for Release {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        write!(
            f,
            "{} ({}), by {}",
            self.title, self.year, self.artists_sort
        )
    }
}

#[cfg(test)]
mod tests {
    //{{{
    use crate::release::Release;

    #[test]
    fn test_release_metadata() {
        // https://www.discogs.com/release/8196883
        let rel = Release::get(8196883).unwrap();
        assert_eq!(rel.year, 1998);
        assert_eq!(rel.uri, "https://www.discogs.com/release/8196883-Monica-Groop-Ostrobothnian-Chamber-Orchestra-Conductor-Juha-Kangas-Bach-Alto-Cantatas");
        assert_eq!(rel.id, 8196883);
        assert_eq!(rel.genres, vec!["Classical"]);
        assert_eq!(rel.artists[0].name, "Monica Groop");
        assert_eq!(rel.get_composers().unwrap(), vec!["Johann Sebastian Bach"]);
    }

    #[test]
    fn test_release_tracklist() {
        let rel = Release::get(8196883).unwrap();
        assert_eq!(rel.tracklist().len(), 19);
        assert_eq!(
            rel.tracklist().first().unwrap().to_string(),
            "[6:22] Aria \"Vergnügte Ruh, Beliebte Seelenlust\"",
        );
        assert_eq!(rel.tracklist().first().unwrap().duration, "6:22");
    }

    #[test]
    fn test_display() {
        // https://www.discogs.com/release/2922014
        let rel = Release::get(2922014).unwrap();
        assert_eq!(
            rel.to_string(),
            "Mating Call (1957), by Tadd Dameron With John Coltrane"
        );
    }

    #[test]
    fn test_no_durations() {
        // https://www.discogs.com/release/2922014
        let rel = Release::get(2922014).unwrap();
        assert_eq!(rel.tracklist().first().unwrap().duration, "");
    }

    #[test]
    fn test_nonexistent_release() {
        assert_eq!(Release::get(0), None);
    }
} //}}}
