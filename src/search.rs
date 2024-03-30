use std::fmt::Display;

use serde::Deserialize;
use serde::Serialize;

use crate::release::Release;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
/// Reusable across Search, Collection, etc.
pub struct Page {
    pub items: usize,
    pub page: usize,
    pub pages: usize,

    /// 50 by default
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
    pub label: Vec<String>, // should be renamed to labels
    /// May be 0, which means it has no master.
    master_id: usize,
    master_url: Option<String>,
    resource_url: String,
    style: Vec<String>,
    thumb: String,
    title: String,
    uri: String,
    // this makes sorting very annoying
    pub year: Option<String>,
    // barcode: Vec [],
    format: Vec<String>,
    // formats: Vec ,
    // user_data: Object {
    //     in_collection: Bool,
    //     in_wantlist: Bool,
    // community: Object {
    //     have: usize,
    //     want: usize,
    // },
}

impl SearchRelease {
    pub fn as_rel(&self) -> Release { Release::get(self.id).unwrap() }
}

impl Display for SearchRelease {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        writeln!(
            f,
            "{} {} {} {}",
            self.year.as_ref().unwrap_or(&"????".to_string()),
            self.id,
            self.format.first().unwrap(),
            self.label.first().unwrap_or(&"No label".to_string()),
        )?;
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

    pub fn remove_no_year(mut self) -> Self {
        self.results.retain(|r| r.year.is_some());
        self
    }

    pub fn sort(
        mut self,
        // sort: SortField,
    ) -> Self {
        fn get_year(s: &SearchRelease) -> usize {
            s.year.as_ref().unwrap_or(&"0".to_string()).parse().unwrap()
        }

        self.results.sort_by_key(get_year);
        self
    }
}

pub enum SortField {
    Year,
    Format,
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
        // assert_eq!(search.results.first().unwrap().id, 1722463);

        // // not the most stable test case
        // let search = search.remove_no_year();
        // assert_eq!(search.results.len(), 32);
        // let search = search.sort();
        // assert_eq!(search.results.len(), 32);

        let pri = search.find_primary().unwrap();
        assert_eq!(pri.id, 377464);
        assert_eq!(pri.year, 1984);
        // assert_eq!(pri.tracklist.len(), 8);
        assert_eq!(pri.tracklist().len(), 8);
    }

    #[test]
    fn test_empty_search() {
        let album = "djsakldjsakl";
        let artist = "metallica";
        assert_eq!(Release::search(artist, album).results.len(), 0);
    }
}
