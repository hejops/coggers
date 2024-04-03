//! Written as an exercise to use basic tree structures for music discovery.

// https://github.com/eliben/code-for-blog/blob/master/2021/rust-bst/src/nodehandle.rs

use std::collections::HashSet;
use std::env;
use std::fmt::Display;

use lazy_static::lazy_static;
use serde::Deserialize;
use serde_json::Value;

lazy_static! {
    pub static ref LASTFM_KEY: String =
        env::var("LASTFM_KEY").expect("Environment variable $LASTFM_KEY must be set");
}

#[derive(Debug)]
pub struct Edge(String, String, f64);

#[derive(Debug)]
pub struct ArtistTree {
    root: String,
    pub edges: Vec<Edge>,
}

impl ArtistTree {
    pub fn new(root: &str) -> Self {
        let root = root.to_string();
        let edges = vec![];
        Self { root, edges }
    }

    // fn contains(
    //     &self,
    //     artist: &SimilarArtist,
    // ) -> bool {
    //     self.nodes.iter().any(|a| a.eq(artist))
    // }

    pub fn build(&mut self) {
        let maxdepth = 1;
        for i in 0..=maxdepth {
            let ch = match i {
                0 => SimilarArtist::new(&self.root).get_edges(),
                _ => {
                    let parents: HashSet<_> =
                        HashSet::from_iter(self.edges.iter().map(|e| e.0.as_str()));

                    // all children
                    HashSet::from_iter(self.edges.iter().map(|e| e.1.as_str()))
                        // minus parents
                        .difference(&parents)
                        .collect::<HashSet<_>>()
                        .iter()
                        // .flat_map(|p| get_children(p))
                        .flat_map(|p| SimilarArtist::new(p).get_edges())
                        .collect::<Vec<Edge>>()
                }
            };
            self.edges.extend(ch);
        }
    }
    // https://depth-first.com/articles/2020/02/03/graphs-in-rust-an-introduction-to-petgraph/
}

impl Display for ArtistTree {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        for edge in self.edges.iter() {
            let Edge(n1, n2, _sim) = edge;
            // let a1 = &self.nodes.get(*n1).unwrap().name;
            // let a2 = &self.nodes.get(*n2).unwrap().name;
            writeln!(f, "{} -> {}", n1, n2)?;
        }
        Ok(())
    }
}

/// This struct is quite poorly implemented
#[derive(Deserialize, Debug)]
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
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            similarity: "1.0".to_string(),
        }
    }

    fn sim_gt(
        &self,
        x: f64,
    ) -> bool {
        self.similarity.parse::<f64>().unwrap() > x
    }

    fn get_similar(&self) -> Vec<SimilarArtist> {
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

    /// Get 1-level children of the node
    fn get_edges(&self) -> Vec<Edge> {
        self.get_similar()
            .into_iter()
            .filter(|c| c.sim_gt(0.7))
            .map(|c| Edge(self.name.to_string(), c.name, c.similarity.parse().unwrap()))
            .collect()
    }
}

impl Display for SimilarArtist {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        write!(f, "{}", self.name)?;
        Ok(())
    }
}
