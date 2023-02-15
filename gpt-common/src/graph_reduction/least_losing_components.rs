use petgraph::{algo::connected_components, prelude::UnGraph};

use crate::{dto::NTupleOutput, graph_reduction::create_graph_url};

use super::{
    common::{clone_with_different_edge_type, join_nodes_on_edge},
    NTupleGraph,
};

fn evaluate_edges_component_count(graph: &mut UnGraph<NTupleOutput, usize>) {
    let initial_component_count = connected_components(&*graph);
    for edge_index in graph.edge_indices() {
        let mut working_graph = graph.clone();
        let (a, b) = working_graph
            .edge_endpoints(edge_index)
            .expect("Edge index should be present in graph");

        join_nodes_on_edge(&mut working_graph, a, b);

        let new_component_count = connected_components(&working_graph);
        graph[edge_index] = new_component_count - initial_component_count;
    }

    log::debug!(
        "Partial data, least losing component graph: \n\n{}",
        create_graph_url(&*graph)
    );
}

pub fn run_least_losing_components(graph: &NTupleGraph) -> NTupleGraph {
    let mut graph: UnGraph<NTupleOutput, usize> = clone_with_different_edge_type(graph);

    while graph.edge_count() > 0 {
        evaluate_edges_component_count(&mut graph);

        let min_index = graph
            .edge_indices()
            .min_by_key(|edge_index| graph[*edge_index])
            .expect("We've checked that we have edges in the graph");
        let (a, b) = graph
            .edge_endpoints(min_index)
            .expect("We should have the min index in the graph");

        join_nodes_on_edge(&mut graph, a, b);
    }

    clone_with_different_edge_type(&graph)
}
