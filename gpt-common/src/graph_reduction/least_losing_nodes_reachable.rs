use std::default;

use petgraph::{
    prelude::{NodeIndex, UnGraph},
    visit::Bfs,
};

use super::{common::clone_with_different_edge_type, NTupleGraph};
use crate::{
    dto::NTupleSingleInterval,
    graph_reduction::{common::join_nodes_on_edge, create_graph_url},
};

fn nodes_reachable(graph: &NTupleGraph<usize>, start_node: NodeIndex) -> usize {
    let mut bfs = Bfs::new(graph, start_node);
    let mut count = 0;

    while bfs.next(graph).is_some() {
        count += 1;
    }

    count
}

fn evaluate_edges_edges_reachable_count(graph: &mut NTupleGraph<usize>) {
    for edge_index in graph.edge_indices() {
        let mut working_graph = graph.clone();
        let (a, b) = working_graph
            .edge_endpoints(edge_index)
            .expect("Edge index should be present in graph");

        let initially_reachable = nodes_reachable(&working_graph, a);
        let joined_ntuple = join_nodes_on_edge(&mut working_graph, a, b);
        let reachable_after_the_join = nodes_reachable(&working_graph, joined_ntuple);

        graph[edge_index] = initially_reachable - reachable_after_the_join;
    }

    log::debug!(
        "Partial data, least losing edges reachable graph: \n\n{}",
        create_graph_url(&*graph)
    );
}

pub fn run_least_losing_edges_reachable<E>(graph: &NTupleGraph<E>) -> NTupleGraph<E>
where
    E: Default,
{
    let mut graph = clone_with_different_edge_type::<E, usize>(graph);

    while graph.edge_count() > 0 {
        evaluate_edges_edges_reachable_count(&mut graph);

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
