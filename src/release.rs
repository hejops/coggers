use serde::Deserialize;
use serde::Serialize;

use crate::http;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Track {
    pub title: String,
    pub duration: String,
    pub type_: String,
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
    pub role: String,
    pub tracks: String, // this being a String is very problematic
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Release {
    pub blocked_from_sale: bool,
    pub id: usize, // u32 is probably fine
    pub year: u16,

    pub data_quality: String, // TODO: enum
    pub status: String,       // TODO: enum

    pub resource_url: String, // api.discogs.com
    pub uri: String,          // www.discogs.com

    pub artists_sort: String,
    pub country: String,
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
    lowest_price: f32,
    num_for_sale: usize,
}

pub fn get_release(release_id: usize) -> Option<Release> {
    let url_fragment = "/releases/".to_string() + release_id.to_string().as_ref();

    let resp = http::make_request(&url_fragment).ok()?;

    match resp.status() {
        reqwest::StatusCode::OK => serde_json::from_str(resp.text().unwrap().as_str()).unwrap(),
        _ => None,
    }
}
