use core::time;
use std::env;

use reqwest::blocking::Response;
use reqwest::header::AUTHORIZATION;
use reqwest::header::CACHE_CONTROL;
use reqwest::header::USER_AGENT;
use serde_json::Value;

// https://www.discogs.com/developers/

const API_PREFIX: &str = "https://api.discogs.com";

// fn get_current_epoch() -> u64 {}

pub struct Credentials {
    username: String,
    token: String,
    timestamps: Vec<u64>,
}
impl Credentials {
    fn build() -> Self {
        let username = env::var("DISCOGS_USERNAME").expect("env var");
        let token = env::var("DISCOGS_TOKEN").expect("env var");
        let timestamps = vec![];

        Credentials {
            username,
            token,
            timestamps,
        }
    }
    fn add_timestamp(&mut self) {
        self.timestamps.push(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        )
    }

    /// Check number of timestamps that fall within the last 60 seconds, and
    /// wait if this number exceeds the limit. Discogs enforces a limit of
    /// 60 requests per minute for authenticated clients.
    fn check_timestamps(&mut self) {
        while self
            .timestamps
            .iter()
            .map(|t| *t - self.timestamps.first().unwrap()) // subtract all elements by the first
            .filter(|&t| t < 60)
            .count()
            >= 60
        {
            std::thread::sleep(time::Duration::from_secs(1));
        }
    }
}

pub enum RequestType {
    Release,
    Artist,
    Label,
    Collection,
    Search,
}

/// Most request types do not require the username, except collection. The
/// request type is specified as we do not expose Credentials.
pub fn make_request(
    request_type: RequestType,
    query: &str,
) -> Result<Response, reqwest::Error> {
    let mut creds = Credentials::build();

    let url_fragment = match request_type {
        RequestType::Collection => format!("/users/{}/{query}", creds.username),
        RequestType::Release => format!("/releases/{query}"),
        RequestType::Search => query.to_string(),
        // artist, label
        _ => unimplemented!(),
    };

    creds.add_timestamp(); // this should be done during/after the request, but that seems non-trivial
    creds.check_timestamps();

    let client = reqwest::blocking::Client::new();
    client
        .get(format!("{}{}", API_PREFIX, url_fragment))
        .header(USER_AGENT, "Discogs client")
        .header(CACHE_CONTROL, "no-cache")
        .header(AUTHORIZATION, format!("Discogs token={}", creds.token))
        .send()
}

/// transform json response to serde Value
pub fn parse_json(resp: Response) -> Value {
    // https://github.com/serde-rs/json?tab=readme-ov-file#parsing-json-as-strongly-typed-data-structures
    // TODO: return type generic (Release, Master, Artist)
    serde_json::from_str(resp.text().unwrap().as_str()).unwrap()
}

// #[test]
// fn test_request_count() {
//     // not sure how to test this yet
// }
