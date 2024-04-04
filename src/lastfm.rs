//! Written as an exercise to use basic tree structures for music discovery.

// https://github.com/eliben/code-for-blog/blob/master/2021/rust-bst/src/nodehandle.rs
use std::collections::HashSet;
use std::env;
use std::f64;
use std::fmt::Display;
use std::process::Command;
use std::process::Stdio;

use anyhow::Context;
use anyhow::Result;
use lazy_static::lazy_static;
use petgraph::dot::Dot;
use petgraph::graph::Graph;
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

    /// Default: 0.7
    threshold: f64,
    /// Default: 2
    depth: u8,
}

impl ArtistTree {
    /// Defaults to threshold 0.7, depth 2
    pub fn new(root: &str) -> Self {
        let root = root.to_string();
        let edges = vec![];
        let threshold = 0.7;
        let depth = 2;
        Self {
            root,
            edges,
            threshold,
            depth,
        }
    }

    pub fn with_threshold(
        mut self,
        new: f64,
    ) -> Self {
        self.threshold = new;
        self
    }

    pub fn with_depth(
        mut self,
        new: u8,
    ) -> Self {
        self.depth = new;
        self
    }

    // fn contains(
    //     &self,
    //     artist: &SimilarArtist,
    // ) -> bool {
    //     self.nodes.iter().any(|a| a.eq(artist))
    // }

    pub fn build(&mut self) {
        for i in 0..=self.depth {
            let ch = match i {
                0 => SimilarArtist::new(&self.root).get_edges(self.threshold),
                _ => {
                    let parents: HashSet<_> =
                        HashSet::from_iter(self.edges.iter().map(|e| e.0.as_str()));
                    let children = HashSet::from_iter(self.edges.iter().map(|e| e.1.as_str()));

                    let nodes: HashSet<_> = parents.union(&children).collect();

                    let children = children
                        .difference(&parents)
                        .collect::<HashSet<_>>()
                        .iter()
                        // damn is this ugly
                        .map(|p| SimilarArtist::new(p).get_edges(self.threshold))
                        .filter(|e| e.is_some())
                        .flat_map(|e| e.unwrap())
                        .filter(|e| !nodes.contains(&e.1.as_str())) // remove cycles
                        .collect::<Vec<Edge>>();
                    Some(children)
                }
            };
            self.edges.extend(ch.unwrap());
        }
    }

    pub fn as_graph(&self) -> Graph<&str, f64> {
        // https://depth-first.com/articles/2020/02/03/graphs-in-rust-an-introduction-to-petgraph/
        let mut graph = Graph::new();
        for edge in self.edges.iter() {
            let Edge(parent, child, sim) = edge;

            // // naive add; leads to node duplication
            // let n1 = graph.add_node(parent.as_str());
            // let n2 = graph.add_node(child.as_str());

            // instead, we should check the graph if either node already exists; if it does,
            // use its NodeIndex
            let n1 = match graph.node_indices().find(|i| graph[*i] == parent) {
                Some(node) => node,
                None => graph.add_node(parent.as_str()),
            };

            let n2 = match graph.node_indices().find(|i| graph[*i] == child) {
                Some(node) => node,
                None => graph.add_node(child.as_str()),
            };

            graph.add_edge(n1, n2, *sim);
        }

        graph
    }

    pub fn as_dot(&self) -> Result<()> {
        // echo {out} | fdp -Tsvg | display

        let g = &self.as_graph();
        let dot = Dot::new(g);
        let echo = Command::new("echo")
            .arg(dot.to_string())
            .stdout(Stdio::piped())
            .spawn()?;
        let fdp = Command::new("fdp")
            .arg("-T")
            .arg("svg")
            .stdin(Stdio::from(echo.stdout.unwrap()))
            .stdout(Stdio::piped())
            .spawn()?;
        Command::new("display")
            .stdin(Stdio::from(fdp.stdout.unwrap()))
            .spawn()?
            .wait()?;

        Ok(())
    }
}

impl Display for ArtistTree {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        for edge in self.edges.iter() {
            let Edge(n1, n2, _sim) = edge;
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

    fn get_similar(&self) -> Result<Vec<SimilarArtist>> {
        let url = format!(
            "http://ws.audioscrobbler.com/2.0/?method=artist.getsimilar&artist={}&api_key={}&format=json", 
            self.name,
            *LASTFM_KEY
        );

        let resp = reqwest::blocking::get(url)?.text()?;
        let raw_json: Value = serde_json::from_str(&resp)?;

        let sim = raw_json
            .get("similarartists")
            .context("no similarartists")?
            .get("artist")
            .unwrap();

        Ok(serde_json::from_value(sim.clone())?)
    }

    /// Get 1-level children of the node. This is done mainly to avoid making
    /// excessive API calls to last.fm.
    fn get_edges(
        &self,
        thresh: f64,
    ) -> Option<Vec<Edge>> {
        match self.get_similar() {
            Ok(similar) => Some(
                similar
                    .into_iter()
                    .filter(|c| c.sim_gt(thresh))
                    .map(|c| Edge(self.name.to_string(), c.name, c.similarity.parse().unwrap()))
                    .collect(),
            ),
            Err(_) => None,
        }
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
