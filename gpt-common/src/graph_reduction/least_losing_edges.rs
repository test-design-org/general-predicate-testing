use std::collections::HashSet;

use petgraph::{
    prelude::{EdgeIndex, NodeIndex},
    visit::EdgeRef,
};

use super::{common::clone_with_different_edge_type, NTupleGraph};
use crate::graph_reduction::common::join_nodes_on_edge;

#[allow(dead_code)]
fn disjunct_neighbours<E>(graph: &NTupleGraph<E>, a: NodeIndex, b: NodeIndex) -> Vec<NodeIndex>
where
    E: Default + Clone,
{
    let a_neighbours = graph.neighbors(a).collect::<HashSet<_>>();
    let b_neighbours = graph.neighbors(b).collect::<HashSet<_>>();

    a_neighbours
        .symmetric_difference(&b_neighbours)
        .copied()
        .collect()
}

#[allow(dead_code)]
fn all_edges_connected_to_disjunt_neighbours<E>(
    graph: &NTupleGraph<E>,
    a: NodeIndex,
    b: NodeIndex,
) -> Vec<EdgeIndex<u32>>
where
    E: Default + Clone + PartialEq,
{
    disjunct_neighbours(graph, a, b)
        .into_iter()
        .flat_map(|node| graph.edges(node).map(|edge_ref| edge_ref.id()))
        .collect::<Vec<_>>()
}

fn disjunct_neighbour_count<E>(graph: &NTupleGraph<E>, a: NodeIndex, b: NodeIndex) -> usize
where
    E: Default + Clone,
{
    let a_neighbours = graph.neighbors(a).collect::<HashSet<_>>();
    let b_neighbours = graph.neighbors(b).collect::<HashSet<_>>();

    let only_neighbour_of_a = a_neighbours.difference(&b_neighbours).count();
    let only_neighbour_of_b = b_neighbours.difference(&a_neighbours).count();

    only_neighbour_of_a + only_neighbour_of_b
}

fn update_edge_weight(graph: &mut NTupleGraph<usize>, edge_index: EdgeIndex) {
    let (a, b) = graph
        .edge_endpoints(edge_index)
        .expect("Edge index should be present in graph");

    graph[edge_index] = disjunct_neighbour_count(graph, a, b);
}

fn evaluate_edges_edge_count(graph: &mut NTupleGraph<usize>) {
    for edge_index in graph.edge_indices() {
        update_edge_weight(graph, edge_index);
    }
}

enum Variant {
    Least,
    Most,
}

// TODO: This should be revised, as it doesn't give the same results as the normal one
#[allow(dead_code)]
fn run_x_losing_edges_optimised<E>(graph: &NTupleGraph<E>, variant: &Variant) -> NTupleGraph<E>
where
    E: Default,
{
    let mut graph = clone_with_different_edge_type::<E, usize>(graph);

    evaluate_edges_edge_count(&mut graph);

    // println!("{}", create_graph_url(&graph));

    while graph.edge_count() > 0 {
        // evaluate_edges_edge_count(&mut graph);

        let selected_edge_index = match variant {
            Variant::Least => graph
                .edge_indices()
                .min_by_key(|edge_index| graph[*edge_index])
                .expect("We've checked that we have edges in the graph"),
            Variant::Most => graph
                .edge_indices()
                .max_by_key(|edge_index| graph[*edge_index])
                .expect("We've checked that we have edges in the graph"),
        };
        let (a, b) = graph
            .edge_endpoints(selected_edge_index)
            .expect("We should have the min index in the graph");

        let edges = all_edges_connected_to_disjunt_neighbours(&(graph.clone()), a, b);

        for edge_index in edges {
            graph[edge_index] = graph[edge_index].saturating_sub(1);
        }

        let joined_node = join_nodes_on_edge(&mut graph, a, b);

        for x in graph.clone().edges(joined_node) {
            update_edge_weight(&mut graph, x.id());
        }

        // evaluate_edges_edge_count(&mut graph);
        // println!("{}", create_graph_url(&graph));
    }

    clone_with_different_edge_type(&graph)
}

fn run_x_losing_edges<E>(graph: &NTupleGraph<E>, variant: Variant) -> NTupleGraph<E>
where
    E: Default,
{
    let mut graph = clone_with_different_edge_type::<E, usize>(graph);

    while graph.edge_count() > 0 {
        evaluate_edges_edge_count(&mut graph);

        let selected_edge_index = match variant {
            Variant::Least => graph
                .edge_indices()
                .min_by_key(|edge_index| graph[*edge_index])
                .expect("We've checked that we have edges in the graph"),
            Variant::Most => graph
                .edge_indices()
                .max_by_key(|edge_index| graph[*edge_index])
                .expect("We've checked that we have edges in the graph"),
        };
        let (a, b) = graph
            .edge_endpoints(selected_edge_index)
            .expect("We should have the min index in the graph");

        join_nodes_on_edge(&mut graph, a, b);
    }

    clone_with_different_edge_type(&graph)
}

pub fn run_least_losing_edges<E>(graph: &NTupleGraph<E>) -> NTupleGraph<E>
where
    E: Default,
{
    run_x_losing_edges(graph, Variant::Least)
}

pub fn run_most_losing_edges<E>(graph: &NTupleGraph<E>) -> NTupleGraph<E>
where
    E: Default,
{
    run_x_losing_edges(graph, Variant::Most)
}
