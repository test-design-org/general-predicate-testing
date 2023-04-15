mod common;
pub mod least_losing_components;
pub mod least_losing_nodes_reachable;
pub mod monke;

use std::fmt::Debug;

use petgraph::{dot::Dot, prelude::UnGraph};
use urlencoding::encode;

use crate::{dto::NTupleSingleInterval, interval::Intersectable};

pub type NTupleGraph = UnGraph<NTupleSingleInterval, ()>;

pub fn create_graph(ntuples: &[NTupleSingleInterval]) -> NTupleGraph {
    let mut graph = UnGraph::<NTupleSingleInterval, ()>::new_undirected();

    for ntuple in ntuples.iter() {
        graph.add_node(ntuple.clone());
    }

    for a in graph.node_indices() {
        for b in graph.node_indices() {
            if a == b
                || graph
                    .edges_connecting(a, b)
                    .chain(graph.edges_connecting(b, a))
                    .count()
                    > 0
            {
                continue;
            }

            let x = graph.node_weight(a).unwrap();
            let y = graph.node_weight(b).unwrap();

            if x.intersects_with(y) {
                graph.add_edge(a, b, ());
            }
        }
    }

    graph
}

pub fn create_graph_url<E>(graph: &UnGraph<NTupleSingleInterval, E>) -> String
where
    E: Debug,
{
    let dot_string = format!("{:?}", Dot::with_config(&graph, &[]));
    let _encoded_dot_string = encode(&dot_string).replace(' ', "%20");

    // "https://dreampuf.github.io/GraphvizOnline/#".to_owned() + &encoded_dot_string
    dot_string
}
