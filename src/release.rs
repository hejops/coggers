use std::vec;

use serde::Deserialize;
use serde::Serialize;

use crate::http;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
enum TrackType {
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
    pub type_: TrackType, // enum
    pub sub_tracks: Option<Vec<Track>>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Label {
    pub id: usize,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Artist {
    // TODO: empty strings should be deserialised to None
    pub anv: String,
    pub id: usize,
    pub name: String,
    pub resource_url: String,
    pub role: String,   // is this an enum? i doubt it
    pub tracks: String, // this being a String is very problematic
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Release {
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
    /// URL in the form www.discogs.com/release/123, equivalent to what would be
    /// displayed in a browser
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
    /// The currency is not specified.
    lowest_price: f32,
    num_for_sale: usize,
}

impl Release {
    pub fn get(release_id: usize) -> Option<Self> {
        let url_fragment = "/releases/".to_string() + release_id.to_string().as_ref();

        let resp = http::make_request(&url_fragment).ok()?;

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
