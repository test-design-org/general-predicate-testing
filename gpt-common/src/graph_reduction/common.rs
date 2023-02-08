use petgraph::{graph::Edge, prelude::NodeIndex, prelude::UnGraph};

use crate::{dto::NTupleOutput, interval::Intersectable};

pub fn replace_nodes<T>(
    graph: &mut UnGraph<NTupleOutput, T>,
    a: NodeIndex,
    b: NodeIndex,
    newNTuple: NTupleOutput,
) where
    T: Default + Clone,
{
    let adjacens_nodes = graph
        .neighbors(a)
        .chain(graph.neighbors(b))
        .filter(|node_index| *node_index != a && *node_index != b)
        .collect::<Vec<NodeIndex>>();

    let ntuple_index = graph.add_node(newNTuple.clone());

    for node_index in adjacens_nodes {
        let node = graph.node_weight(node_index).unwrap();
        if newNTuple.intersects_with(node) {
            graph.add_edge(ntuple_index, node_index, T::default());
        }
    }
    graph.remove_node(a);
    graph.remove_node(b);
}

pub fn join_nodes_on_edge<T>(
    graph: &mut UnGraph<NTupleOutput, T>,
    a: NodeIndex,
    b: NodeIndex,
) -> NTupleOutput
where
    T: Default + Clone,
{
    let joined_ntuple = graph
        .node_weight(a)
        .unwrap()
        .intersect(graph.node_weight(b).unwrap())
        .unwrap();

    replace_nodes(graph, a, b, joined_ntuple.clone());

    joined_ntuple
}

// export function minimumBy<T>(list: T[], lens: (_: T) => number): T {
//   return list.reduce((minimum, x) => (lens(x) < lens(minimum) ? x : minimum));
// }

// export function numberOfConnectedComponentsComponents(graph: Graph): number {
//   const visited = new Map<string, boolean>();

//   const DFSUtil = (node: NTuple) => {
//     visited.set(node.id, true);

//     const neighbours = graph.getNeighbours(node);
//     for (const neighbour of neighbours) {
//       if (!visited.get(neighbour.id)) {
//         DFSUtil(neighbour);
//       }
//     }
//   };

//   let componentCount = 0;
//   for (const node of graph.nodes) {
//     if (!visited.get(node.id)) {
//       DFSUtil(node);
//       ++componentCount;
//     }
//   }

//   return componentCount;
// }

// export function dfs(
//   graph: Graph,
//   startingNode: NTuple,
//   events: {
//     discoverNode?: (node: NTuple) => void;
//   },
// ): void {
//   const visited = new Map<string, boolean>();

//   const DFSUtil = (node: NTuple) => {
//     visited.set(node.id, true);
//     events.discoverNode?.(node);

//     const neighbours = graph.getNeighbours(node);
//     for (const neighbour of neighbours) {
//       if (!visited.get(neighbour.id)) {
//         DFSUtil(neighbour);
//       }
//     }
//   };

//   DFSUtil(startingNode);
// }
