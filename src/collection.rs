use serde::Deserialize;
use serde::Serialize;
use serde_json::Result;

use crate::http;
use crate::release::Artist;
use crate::release::Label;
use crate::search::Page;

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
    // TODO: flatten
    // #[serde(flatten)] counterintuitively, flatten actually -increases- nesting
    // basic_information: HashMap<String, Value>,
    pub basic_information: CollectionRelease,
}

/// Contains zero or more CollectionResult, each of which contains one
/// CollectionRelease.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct CollectionResults {
    pub pagination: Page,
    pub releases: Vec<CollectionResult>,
}

impl CollectionResults {
    pub fn dump_collection() -> Result<Self> {
        let i = 1;
        let url = format!("/collection/folders/0/releases?per_page=250&page={i}");
        let resp = http::make_request(http::RequestType::Collection, &url).unwrap();
        serde_json::from_str(resp.text().unwrap().as_str())
    }
}

#[cfg(test)]
mod tests {
    use crate::collection;

    #[test]
    fn test_collection() {
        let coll = collection::CollectionResults::dump_collection().unwrap();
        let r = coll.releases.first().unwrap();
        assert_eq!(r.id, r.basic_information.id);
    }
}
