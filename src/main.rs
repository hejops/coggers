use discogs::http;
use serde_json::Value;

fn main() {
    let resp = http::make_request("/releases/8196883").unwrap();
    let v: Value = serde_json::from_str(resp.text().unwrap().as_str()).unwrap();
    println!("{:#?}", v);
}
