use petgraph::prelude::NodeIndex;

use super::NTupleGraph;
use crate::{dto::NTupleSingleInterval, interval::Intersectable};

pub fn replace_nodes<E>(
    graph: &mut NTupleGraph<E>,
    a: NodeIndex,
    b: NodeIndex,
    new_ntuple: &NTupleSingleInterval,
) -> NodeIndex
where
    E: Default + Clone,
{
    let adjacens_nodes = graph
        .neighbors(a)
        .chain(graph.neighbors(b))
        .filter(|node_index| *node_index != a && *node_index != b)
        // .unique() // TODO: In some examples if we disable it'll reduce more, which is weird
        .collect::<Vec<NodeIndex>>();

    let ntuple_index = graph.add_node(Box::new(new_ntuple.clone()));

    for node_index in adjacens_nodes {
        let node = graph.node_weight(node_index).unwrap();
        if new_ntuple.intersects_with(node) {
            graph.add_edge(ntuple_index, node_index, E::default());
        }
    }
    graph.remove_node(a);
    graph.remove_node(b);

    let ntuple_index = graph
        .node_indices()
        .find(|node_index| *graph[*node_index] == *new_ntuple)
        .expect("The recently added new ntuple should be in the graph");

    ntuple_index
}

pub fn join_nodes_on_edge<E>(graph: &mut NTupleGraph<E>, a: NodeIndex, b: NodeIndex) -> NodeIndex
where
    E: Default + Clone,
{
    let joined_ntuple = graph
        .node_weight(a)
        .unwrap()
        .intersect(graph.node_weight(b).unwrap())
        .unwrap();

    let joined_ntuple_index = replace_nodes(graph, a, b, &joined_ntuple);

    joined_ntuple_index
}

pub fn clone_with_different_edge_type<EOld, ENew>(graph: &NTupleGraph<EOld>) -> NTupleGraph<ENew>
where
    ENew: Default,
{
    let mut new_graph = NTupleGraph::<ENew>::default();

    for node in graph.node_weights() {
        new_graph.add_node(node.clone());
    }

    for edge_index in graph.edge_indices() {
        let (a, b) = graph.edge_endpoints(edge_index).expect(
            "We're iterating through the edge indicies, this index should exist in the graph",
        );

        new_graph.add_edge(a, b, ENew::default());
    }

    new_graph
}
