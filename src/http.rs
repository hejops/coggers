use std::env;

use reqwest::blocking::Response;
use reqwest::header::AUTHORIZATION;
use reqwest::header::CACHE_CONTROL;
use reqwest::header::USER_AGENT;
use serde_json::Value;

const API_PREFIX: &str = "https://api.discogs.com";

struct Credentials {
    // username: String,
    token: String,
}
impl Credentials {
    fn build() -> Self {
        // let username = env::var("DISCOGS_USERNAME").expect("env var");
        let token = env::var("DISCOGS_TOKEN").expect("env var");

        Credentials {
            // username,
            token,
        }
    }
}

pub fn make_request(url_fragment: &str) -> Result<reqwest::blocking::Response, reqwest::Error> {
    let creds = Credentials::build();

    let client = reqwest::blocking::Client::new();
    client
        .get(API_PREFIX.to_string() + url_fragment)
        .header(USER_AGENT, "Discogs client")
        .header(CACHE_CONTROL, "no-cache")
        .header(AUTHORIZATION, "Discogs token=".to_string() + &creds.token)
        .send()
}

/// transform json response to serde Value
pub fn parse_json(resp: Response) -> Value {
    // https://github.com/serde-rs/json?tab=readme-ov-file#parsing-json-as-strongly-typed-data-structures
    // TODO: return type generic (Release, Master, Artist)
    serde_json::from_str(resp.text().unwrap().as_str()).unwrap()
}
