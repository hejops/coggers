use std::env;

use reqwest::header::AUTHORIZATION;
use reqwest::header::CACHE_CONTROL;
use reqwest::header::USER_AGENT;

pub const API_PREFIX: &str = "https://api.discogs.com";

struct Credentials {
    username: String,
    token: String,
}
impl Credentials {
    fn build() -> Self {
        let username = env::var("DISCOGS_USERNAME").unwrap();
        let token = env::var("DISCOGS_TOKEN").unwrap();

        Credentials { username, token }
    }
}

pub fn make_request(url_fragment: &str) -> Result<reqwest::blocking::Response, reqwest::Error> {
    let creds = Credentials::build();

    // reqwest::blocking::get(url)

    let client = reqwest::blocking::Client::new();
    client
        .get(API_PREFIX.to_string() + url_fragment)
        .header(USER_AGENT, "Discogs client")
        .header(CACHE_CONTROL, "no-cache")
        .header(AUTHORIZATION, "Discogs token=".to_string() + &creds.token)
        .send()
}
