pub mod http; // re-export for main, and to silence dead_code

#[cfg(test)]
mod tests {
    #[test]
    fn test_request() {
        use crate::http;

        // https://www.discogs.com/release/8196883
        let resp = http::make_request("/releases/8196883").unwrap();

        assert_eq!(resp.status(), 200);

        let json = http::parse_json(resp);

        assert_eq!(json["year"], 1998);
        assert_eq!(json["id"], 8196883);
    }
}
