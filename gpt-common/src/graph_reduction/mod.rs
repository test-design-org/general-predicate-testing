use std::fmt::Display;

use petgraph::{
    dot::{Config, Dot},
    prelude::UnGraph,
    visit::{IntoEdgeReferences, IntoNodeReferences},
    Graph,
};
use urlencoding::encode;

use crate::{dto::NTupleOutput, interval::Intersectable};

pub type NTupleGraph = UnGraph<NTupleOutput, ()>;

pub fn create_graph(ntuples: &[NTupleOutput]) -> NTupleGraph {
    let mut graph = UnGraph::<NTupleOutput, ()>::new_undirected();

    for ntuple in ntuples.iter() {
        graph.add_node(ntuple.clone());
    }

    for a in graph.node_indices() {
        for b in graph.node_indices() {
            let x = graph.node_weight(a).unwrap();
            let y = graph.node_weight(b).unwrap();

            if x.intersects_with(y) {
                graph.add_edge(a, b, ());
            }
        }
    }

    graph
}

pub fn create_graph_url(graph: &NTupleGraph) -> String {
    let dot_string = format!("{:?}", Dot::with_config(&graph, &[Config::EdgeNoLabel]));
    let encoded_dot_string = encode(&dot_string).into_owned();

    "https://dreampuf.github.io/GraphvizOnline/#".to_owned() + &encoded_dot_string
}
