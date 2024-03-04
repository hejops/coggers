use serde::Deserialize;
use serde::Serialize;

#[derive(Serialize, Deserialize)]
pub struct Release {
    pub id: u32,
    pub uri: String,
    pub year: u32,
}
