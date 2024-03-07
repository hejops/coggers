use serde::Deserialize;
use serde::Serialize;

use crate::http;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct SearchPage {
    pub items: usize,
    pub page: usize,
    pub pages: usize,
    pub per_page: usize,
    // "urls": Object {
    //     "last": String,
    //     "next": String,
    // },
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
/// Release from a search. This is distinct from Release, notably due to the
/// absence of tracklist.
pub struct ReleaseResult {
    r#type: String,
    catno: String,
    country: String,
    cover_image: String,
    format_quantity: usize,
    genre: Vec<String>,
    id: usize,
    label: Vec<String>,
    master_id: usize,
    master_url: Option<String>,
    resource_url: String,
    style: Vec<String>,
    thumb: String,
    title: String,
    uri: String,
    // barcode: Vec [],
    // format: Vec ,
    // formats: Vec ,
    // user_data: Object {
    //     in_collection: Bool,
    //     in_wantlist: Bool,
    // community: Object {
    //     have: usize,
    //     want: usize,
    // },
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct SearchResults {
    pub pagination: SearchPage,
    /// The results field will always exist, but may be empty.
    pub results: Vec<ReleaseResult>,
}

pub fn search_release(
    artist: &str,
    album: &str,
) -> Option<Vec<ReleaseResult>> {
    let url = format!("/database/search?release_title={album}&artist={artist}&type=release");

    let resp = http::make_request(&url).unwrap();
    let results: SearchResults = serde_json::from_str(resp.text().unwrap().as_str()).unwrap();
    // https://stackoverflow.com/a/65012849
    (!results.results.is_empty()).then_some(results.results)
}
