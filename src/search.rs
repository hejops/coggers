use serde::Deserialize;
use serde::Serialize;

use crate::http;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
/// Reusable across Search, Collection, etc.
pub struct Page {
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
pub struct SearchRelease {
    r#type: String,
    catno: String,
    country: String,
    cover_image: String,
    format_quantity: usize,
    genre: Vec<String>,
    pub id: usize,
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
    pub pagination: Page,
    /// The results field will always exist, but may be empty.
    pub results: Vec<SearchRelease>,
}

#[cfg(test)]
mod tests {
    use crate::release::Release;
    #[test]
    fn test_big_search() {
        let album = "ride the lightning";
        let artist = "metallica";
        assert_eq!(Release::search(artist, album).unwrap().len(), 50);
        assert_eq!(
            Release::search(artist, album).unwrap().first().unwrap().id,
            1722463
        );
    }

    #[test]
    fn test_empty_search() {
        let album = "djsakldjsakl";
        let artist = "metallica";
        assert_eq!(Release::search(artist, album), None);
    }
}
