//! Written as an exercise to use basic tree structures for music discovery.

// the graph was initially implemented as a naive Vec<String> + Vec<(usize,
// usize)> -- see https://github.com/eliben/code-for-blog/blob/master/2021/rust-bst/src/nodehandle.rs.
// eventually i found Vec indexing really annoying, and switched over to an
// "edge-only" Vec<Edge>. then i also found -that- ugly, and switched to a
// HashMap.

use std::collections::HashMap;
use std::env;
use std::f64;
use std::process::Command;
use std::process::Stdio;

use anyhow::Context;
use anyhow::Result;
use lazy_static::lazy_static;
use petgraph::dot::Dot;
use petgraph::graph::Graph;
use petgraph::graph::NodeIndex;
use serde::Deserialize;
use serde_json::Value;

lazy_static! {
    static ref LASTFM_KEY: String =
        env::var("LASTFM_KEY").expect("Environment variable $LASTFM_KEY must be set");
}

#[derive(Debug)]
pub struct Edge(String, String, f64);

#[derive(Debug)]
/// This should be implemented as a tree, because graphs will usually produce
/// many uninteresting cycles.
pub struct ArtistTree {
    root: String,

    // edges: Vec<Edge>,
    nodes: HashMap<String, NodeIndex>,

    /// Default: 0.7
    threshold: f64,

    /// Default: 2
    depth: u8,
}

impl ArtistTree {
    /// Defaults to threshold 0.7, depth 2
    pub fn new(root: &str) -> Self {
        let root = root.to_string().to_lowercase();
        // let edges = vec![];
        let nodes = HashMap::new();
        let threshold = 0.7;
        let depth = 2;
        Self {
            root,
            // edges,
            nodes,
            threshold,
            depth,
        }
    }

    fn with_threshold(
        mut self,
        new: f64,
    ) -> Self {
        self.threshold = new;
        self
    }

    fn with_depth(
        mut self,
        new: u8,
    ) -> Self {
        self.depth = new;
        self
    }

    /// HashMap and Graph are constructed in parallel
    fn build(&mut self) -> Graph<String, String> {
        let mut graph = Graph::new();

        let root = self.root.to_lowercase();
        let r = graph.add_node(root.clone());
        self.nodes.insert(root.clone(), r);

        for _ in 0..=self.depth {
            for parent in self.nodes.clone().keys().map(|p| p.to_lowercase()) {
                let children = match SimilarArtist::new(&parent).get_similar() {
                    Ok(ch) => ch
                        .into_iter()
                        .map(|mut a| {
                            a.name.make_ascii_lowercase();
                            a
                        })
                        .filter(|a| a.sim_gt(0.7)),
                    Err(_) => continue,
                };
                for c in children {
                    let n1 = match self.nodes.get(&parent) {
                        Some(node) => *node,
                        None => graph.add_node(parent.to_string()),
                    };
                    let n2 = match self.nodes.get(&c.name) {
                        Some(_) => continue,
                        None => graph.add_node(c.name.clone()),
                    };
                    graph.add_edge(n1, n2, c.similarity);

                    self.nodes.insert(parent.clone(), n1);
                    self.nodes.insert(c.name, n2);
                }
            }
        }

        graph
    }

    // old Vec<Edge> implementation

    // pub fn build(&mut self) {
    //     for i in 0..=self.depth {
    //         let ch = match i {
    //             0 => SimilarArtist::new(&self.root).get_edges(self.threshold),
    //             _ => {
    //                 let parents: HashSet<_> =
    //                     HashSet::from_iter(self.edges.iter().map(|e|
    // e.0.as_str()));                 let children =
    // HashSet::from_iter(self.edges.iter().map(|e| e.1.as_str()));
    //
    //                 let nodes: HashSet<_> = parents.union(&children).collect();
    //
    //                 let children = children
    //                     .difference(&parents)
    //                     .collect::<HashSet<_>>()
    //                     .iter()
    //                     .map(|p| SimilarArtist::new(p).get_edges(self.threshold))
    //                     .filter(|e| e.is_some())
    //                     .flat_map(|e| e.unwrap())
    //                     // remove cycles
    //                     .filter(|e| !nodes.contains(&e.1.as_str()))
    //                     .collect::<Vec<Edge>>();
    //                 Some(children)
    //             }
    //         };
    //         self.edges.extend(ch.unwrap());
    //     }
    // }

    // fn as_graph(&self) -> Graph<&str, f64> {
    //     // https://depth-first.com/articles/2020/02/03/graphs-in-rust-an-introduction-to-petgraph/
    //     let mut graph = Graph::new();
    //     for edge in self.edges.iter() {
    //         let Edge(parent, child, sim) = edge;
    //
    //         let n1 = match graph.node_indices().find(|i| graph[*i] == parent) {
    //             Some(node) => node,
    //             None => graph.add_node(parent.as_str()),
    //         };
    //
    //         let n2 = match graph.node_indices().find(|i| graph[*i] == child) {
    //             Some(node) => node,
    //             None => graph.add_node(child.as_str()),
    //         };
    //
    //         graph.add_edge(n1, n2, *sim);
    //     }
    //
    //     graph
    // }

    pub fn as_dot(
        &mut self,
        fmt: DotOutput,
    ) -> Result<()> {
        // echo {out} | <fdp|dot> -Tsvg | display

        let g = &self.build();
        let dot = Dot::new(g);
        let ext = match fmt {
            DotOutput::Png => "png",
            DotOutput::Svg => "svg",
        };
        let f = format!("{}.{}", self.root, ext);

        let echo = Command::new("echo")
            .arg(dot.to_string())
            .stdout(Stdio::piped())
            .spawn()?;
        let _fdp = Command::new("dot")
            .args(["-T", ext])
            .stdin(Stdio::from(echo.stdout.unwrap()))
            // .stdout(Stdio::piped())
            .args(["-o", &f])
            .spawn()?
            .wait()?;

        Command::new("display")
            // .stdin(Stdio::from(fdp.stdout.unwrap()))
            .arg(f)
            .spawn()?
            .wait()?;

        Ok(())
    }
}

pub enum DotOutput {
    Png,
    Svg,
}

/// This struct is quite poorly implemented
#[derive(Deserialize, Debug, Clone)]
pub struct SimilarArtist {
    name: String,
    /// Preserved as `String`, in order to be able to implement `Eq`
    #[serde(rename = "match")]
    similarity: String,
}

impl SimilarArtist {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string().to_lowercase(),
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

        Ok(serde_json::from_value::<Vec<SimilarArtist>>(sim.clone())?)
    }

    // /// Get 1-level children of the node. This is done mainly to avoid making
    // /// excessive API calls to last.fm.
    // fn get_edges(
    //     &self,
    //     thresh: f64,
    // ) -> Option<Vec<Edge>> {
    //     match self.get_similar() {
    //         Ok(similar) => Some(
    //             similar
    //                 .iter()
    //                 .filter(|c| c.sim_gt(thresh))
    //                 .map(|c| {
    //                     Edge(
    //                         self.name.to_string().to_lowercase(),
    //                         c.name.to_lowercase(),
    //                         c.similarity.parse().unwrap(),
    //                     )
    //                 })
    //                 .collect(),
    //         ),
    //         Err(_) => None,
    //     }
    // }
}
