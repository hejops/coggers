use serde::Deserialize;
use serde::Serialize;

use crate::http;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Release {
    pub id: u32,
    pub uri: String,
    pub year: u32,
}

pub fn get_release(release_id: usize) -> Option<Release> {
    let url_fragment = "/releases/".to_string() + release_id.to_string().as_ref();

    let resp = http::make_request(&url_fragment).ok()?;

    match resp.status() {
        reqwest::StatusCode::OK => serde_json::from_str(resp.text().unwrap().as_str()).unwrap(),
        _ => None,
    }
}
