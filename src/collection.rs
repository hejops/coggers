use serde::Deserialize;
use serde::Serialize;

use crate::http;
use crate::release::Artist;
use crate::release::Label;
// use crate::search::Page;

/// Does not contain tracklist.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct CollectionRelease {
    /// Equivalent to CollectionResult.id
    pub id: usize,
    master_id: usize,
    artists: Vec<Artist>,
    genres: Vec<String>,
    labels: Vec<Label>,
    master_url: Option<String>,
    resource_url: String,
    styles: Vec<String>,
    title: String,
    year: usize,
    //     formats: Array ,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct CollectionResult {
    pub id: usize,
    instance_id: usize,
    rating: u8,
    date_added: String,

    // TODO: flatten basic_information, i.e. bring CollectionRelease's fields into CollectionResult

    // #[serde(flatten)] counterintuitively, flatten actually -increases- nesting
    // basic_information: HashMap<String, Value>,
    pub basic_information: CollectionRelease,
}

/// Contains zero or more `CollectionResult`s, each of which contains one
/// `CollectionRelease`.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
// #[serde(transparent)]
pub struct Collection {
    // ignoring pagination is a (poor) hack to reuse this json struct for our own purposes; we
    // don't actually ever check it
    // pub pagination: Page,
    pub releases: Vec<CollectionResult>,

    /// This is only for testing, and should never be needed in real use
    #[serde(skip)]
    start_page: usize,
}

impl Default for Collection {
    fn default() -> Self {
        Self {
            releases: vec![],
            start_page: 1,
        }
    }
}

impl Collection {
    pub fn new() -> Self { Self::default() }

    pub fn with_start(
        mut self,
        page: usize,
    ) -> Self {
        self.start_page = page;
        self
    }

    /// Returns Err if current page exceeded the allowed range
    pub fn dump(&self) -> anyhow::Result<Self> {
        let mut releases = vec![];

        let mut i = self.start_page;
        loop {
            let url = format!("/collection/folders/0/releases?per_page=250&page={i}");
            let resp = http::make_request(http::RequestType::Collection, &url)?;
            match serde_json::from_str::<Collection>(resp.text()?.as_str()) {
                Ok(mut coll) => {
                    // println!("{} {}", i, coll.releases.len());
                    releases.append(&mut coll.releases);
                    i += 1;
                }
                Err(_) => break,
            }
        }

        Ok(Collection {
            releases,
            start_page: 1,
        })
        // TODO: transform into sql
    }
}

#[cfg(test)]
mod tests {
    use crate::collection::Collection;

    #[test]
    fn test_collection() {
        let coll = Collection::new().with_start(27).dump().unwrap();
        let r = coll.releases.first().unwrap();
        assert_eq!(r.id, r.basic_information.id);
        assert_eq!(coll.releases.len(), 163);
    }
}
