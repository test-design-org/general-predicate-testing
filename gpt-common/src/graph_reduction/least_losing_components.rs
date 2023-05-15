use petgraph::{algo::connected_components, prelude::UnGraph};

use super::{
    common::{clone_with_different_edge_type, join_nodes_on_edge},
    NTupleGraph,
};
use crate::{dto::NTupleSingleInterval, graph_reduction::create_graph_url};

fn evaluate_edges_components_count(graph: &mut NTupleGraph<usize>) {
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

    #[cfg(debug_assertions)]
    log::debug!(
        "Partial data, least losing components graph: \n\n{}",
        create_graph_url(&*graph)
    );
}

pub fn run_least_losing_components<E>(graph: &NTupleGraph<E>) -> NTupleGraph<E>
where
    E: Default,
{
    let mut graph = clone_with_different_edge_type::<E, usize>(graph);

    while graph.edge_count() > 0 {
        evaluate_edges_components_count(&mut graph);

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
