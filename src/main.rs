use discogs::http;
use discogs::release::Release;
use serde_json::Value;

fn main() {
    let resp = http::make_request("/releases/8196883").unwrap();
    let v: Release = serde_json::from_str(resp.text().unwrap().as_str()).unwrap();
    // println!("{:#?}", v.tracklist);
    let tl = v.parse_tracklist();
    println!("{}", tl.len());
}
