mod http;

#[cfg(test)]
#[test]
fn test_request() {
    // https://www.discogs.com/release/8196883

    assert_eq!(
        http::make_request("/releases/8196883").unwrap().status(),
        200
    );
}
