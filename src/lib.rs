pub mod http; // re-export for main, and to silence dead_code
pub mod release;

#[cfg(test)]
mod tests {
    #[test]
    fn test_request() {
        use crate::http;
        use crate::release::Release;

        // https://www.discogs.com/release/8196883
        let resp = http::make_request("/releases/8196883").unwrap();

        assert_eq!(resp.status(), 200);

        let rel: Release = serde_json::from_str(resp.text().unwrap().as_str()).unwrap();
        assert_eq!(rel.year, 1998);
        assert_eq!(rel.id, 8196883);
    }
}
