use super::{common::join_nodes_on_edge, NTupleGraph};

pub fn run_MONKE(graph: &NTupleGraph) -> NTupleGraph {
    let mut graph = graph.clone();

    while graph.edge_count() > 0 {
        let edge_index = graph.edge_indices().next().unwrap();
        let (a, b) = graph.edge_endpoints(edge_index).unwrap();

        join_nodes_on_edge(&mut graph, a, b);
    }

    graph
}
