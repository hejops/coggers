//! Written as an exercise to implement tree/graph-like structures

// https://github.com/eliben/code-for-blog/blob/master/2021/rust-bst/src/nodehandle.rs

use std::env;

use lazy_static::lazy_static;
use serde::Deserialize;
use serde_json::Value;

// from a given artist (root node), construct a tree of artists, limiting to
// similarity value above 0.5. at each branch, similarity value is multiplied
// with that of the parent node

// response:
// {'similarartists': {'artist': [...]},
//                     '@attr': {'name': foo}}
//
// desired struct:
// {'similarartists': [...],
//  'name': foo}
//
//  i.e.: bring 'name' 2 levels up, bring 'artist' 1 level up

lazy_static! {
    pub static ref LASTFM_KEY: String =
        env::var("LASTFM_KEY").expect("Environment variable $LASTFM_KEY must be set");
}

#[derive(Debug)]
struct Edge((usize, usize));

#[derive(Debug)]
pub struct ArtistTree {
    root: SimilarArtist,
    pub nodes: Vec<SimilarArtist>,
    edges: Vec<Edge>,
}

impl ArtistTree {
    pub fn new(root: SimilarArtist) -> Self {
        let nodes = vec![root.clone()];
        let edges = vec![];
        Self { root, nodes, edges }
    }

    fn contains(
        &self,
        artist: &SimilarArtist,
    ) -> bool {
        self.nodes.iter().any(|a| a.eq(artist))
    }

    pub fn build(&mut self) {
        let maxdepth = 1;
        for i in 0..=maxdepth {
            // we want to extend self.nodes while iterating through it. without using
            // container types like RefCell, two 'naive' options appear viable:
            // self.nodes.clone().into_iter(), or self.nodes.iter(). for memory
            // safety, the latter is disallowed by the borrow checker.
            //
            // TODO: consider wrapping Vec in RefCell to allow interior mutability?
            // https://stackoverflow.com/a/30967250

            for parent in self.nodes.clone() {
                if i > 1 && self.contains(&parent) {
                    continue;
                }

                let mut new = vec![]; // self.nodes is mutable in this block...

                // println!("{}", parent.name);
                for c in parent
                    .get_similar()
                    .iter()
                    .filter(|c| c.sim_gt(0.6) && !self.contains(c))
                {
                    // ...but immutable in this one
                    new.push(c.clone());
                }

                self.nodes.extend(new);
            }
            println!("{} {}", i, self.nodes.len());
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct SimilarArtist {
    pub name: String,
    /// Preserved as `String`, in order to be able to implement `Eq`
    #[serde(rename = "match")]
    similarity: String,
}

impl PartialEq for SimilarArtist {
    fn eq(
        &self,
        other: &Self,
    ) -> bool {
        self.name == other.name
    }
}

impl SimilarArtist {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            similarity: "1.0".to_string(),
        }
    }

    pub fn sim_gt(
        &self,
        x: f64,
    ) -> bool {
        self.similarity.parse::<f64>().unwrap() > x
    }

    /// Get 1-level children of the node
    pub fn get_similar(&self) -> Vec<SimilarArtist> {
        let url = format!(
            "http://ws.audioscrobbler.com/2.0/?method=artist.getsimilar&artist={}&api_key={}&format=json", 
            self.name,
            *LASTFM_KEY
        );

        let resp = reqwest::blocking::get(url).unwrap().text().unwrap();
        let raw_json: Value = serde_json::from_str(&resp).unwrap();

        let sim = raw_json
            .get("similarartists")
            .unwrap()
            .get("artist")
            .unwrap();
        serde_json::from_value(sim.clone()).unwrap()
    }
}

// 200 LOC for a struct with 2 fields is crazy...
// https://serde.rs/deserialize-struct.html

// // https://stackoverflow.com/a/75684771
// // https://github.com/serde-rs/serde/issues/868#issuecomment-520511656
// fn extract_object_generic<'de, D, T>(deserializer: D) -> Result<T, D::Error>
// where
//     D: serde::de::Deserializer<'de>,
//     T: Deserialize<'de>,
// {
//     #[derive(Deserialize)]
//     struct Container<T> {
//         object: T,
//     }
//     Container::deserialize(deserializer).map(|a| a.object)
// }
