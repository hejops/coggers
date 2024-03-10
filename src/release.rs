use serde::Deserialize;
use serde::Serialize;

use crate::http;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum TrackType {
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
    pub duration: String,
    pub type_: TrackType,
    pub sub_tracks: Option<Vec<Track>>,
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

/// The definitive representation of a release, and the only one with tracklist.
/// Similar to CollectionRelease and SearchRelease, both of which contain less
/// information.
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
    pub tracklist: Vec<Track>,

    // companies: Vec,
    // formats: Vec,
    // identifiers: Vec,
    /// The currency is assumed from the locale and not specified in the
    /// response.
    lowest_price: f32,
    num_for_sale: usize,
}

impl Release {
    pub fn get(release_id: usize) -> Option<Self> {
        let resp = http::make_request(http::RequestType::Release, &release_id.to_string()).ok()?;

        match resp.status() {
            reqwest::StatusCode::OK => serde_json::from_str(resp.text().unwrap().as_str()).unwrap(),
            _ => None,
        }
    }

    pub fn parse_tracklist(&self) -> Vec<&Track> {
        pub fn recurse(tracks: &[Track]) -> Vec<&Track> {
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
}

#[cfg(test)]
mod tests {
    //{{{
    use crate::release::Release;

    #[test]
    fn test_release() {
        // https://www.discogs.com/release/8196883
        let rel = Release::get(8196883).unwrap();
        assert_eq!(rel.year, 1998);
        assert_eq!(rel.uri, "https://www.discogs.com/release/8196883-Monica-Groop-Ostrobothnian-Chamber-Orchestra-Conductor-Juha-Kangas-Bach-Alto-Cantatas");
        assert_eq!(rel.id, 8196883);
        assert_eq!(rel.genres, vec!["Classical"]);
        assert_eq!(rel.artists[0].name, "Monica Groop");

        assert_eq!(rel.parse_tracklist().len(), 19);

        assert_eq!(
            rel.parse_tracklist()
                .iter()
                .map(|t| t.title.to_string())
                .collect::<Vec<String>>(),
            vec![
                r#"Aria "Vergnügte Ruh, Beliebte Seelenlust""#,
                r#"Recitativo "Die Welt, Das Sündenhaus""#,
                r#"Aria "Wie Jammern Mich Soch Die Werkehren Herzen""#,
                r#"Recitativo "Wer Sollte Sich Demnach Wohl Hier Zu Leben Wünschen""#,
                r#"Aria "Mir Ekelt Mehr Zu Leben""#,
                r#"Sinfonia"#,
                r#"Aria "Geist Und Seele Wird Verwirret""#,
                r#"Recitativo "Ich Wundre Mich""#,
                r#"Aria "Gott Hat Alles Wohlgemacht""#,
                r#"Sinfonia"#,
                r#"Recitativo "Ach, Starker Gott""#,
                r#"Aria "Ich Wünsche Nur, Bei Gott Zu Leben""#,
                r#"Sinfonia"#,
                r#"Arioso And Recitativo "Gott Soll Alein Mein Herze Haben""#,
                r#"Aria "Gott Soll Alein Mein Herze Haben""#,
                r#"Recitativo "Was Ist Die Liebe Gottes?""#,
                r#"Aria "Stirb In Mir, Welt Und Alle Deine Leibe""#,
                r#"Recitativo "Doch Meint Es Auch Dabei""#,
                r#"Chorale "Du Süsse Liebe, Schenk Uns Deine Gunst""#,
            ]
        );
    }

    #[test]
    fn test_nonexistent_release() {
        assert_eq!(Release::get(0), None);
    }
} //}}}
