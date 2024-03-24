use std::fmt::Display;

use serde::Deserialize;
use serde::Serialize;

use crate::http;
use crate::release::Master;
use crate::release::Release;

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
    /// May be 0, which means it has no master.
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

impl Display for SearchRelease {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        writeln!(f, "{}", self.master_id)?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct SearchResults {
    pub pagination: Page,
    /// The `results` field will always exist in the response, but may be empty.
    pub results: Vec<SearchRelease>,
}

impl SearchResults {
    /// Find first primary release, if a master release exists
    // TODO: fallback to first release? (i.e. first primary -> first release ->
    // None)
    pub fn find_primary(&self) -> Option<Release> {
        for res in &self.results {
            if res.master_id > 0 {
                let m = Release::get_master(res.master_id).unwrap();
                return Release::get(m.main_release);
            }
        }
        None
    }
}

impl Display for SearchResults {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        for res in &self.results {
            write!(f, "{}", res)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::release::Release;
    #[test]
    fn test_big_search() {
        let album = "ride the lightning";
        let artist = "metallica";
        let search = Release::search(artist, album);
        assert_eq!(search.results.len(), 50);
        assert_eq!(search.results.first().unwrap().id, 1722463);
        assert_eq!(search.find_primary().unwrap().id, 377464);
    }

    #[test]
    fn test_empty_search() {
        let album = "djsakldjsakl";
        let artist = "metallica";
        assert_eq!(Release::search(artist, album).results.len(), 0);
    }
}
