import { union } from 'fp-ts/lib/Array';
import { Edge, Graph, NTuple } from '@testing-repo/gpt-common';

export function replaceNodes(
  graph: Graph,
  a: NTuple,
  b: NTuple,
  newNTuple: NTuple,
) {
  const adjacentNodes = union(NTuple.Eq)(
    graph.getNeighbours(a),
    graph.getNeighbours(b),
  ).filter((x) => !NTuple.Eq.equals(x, a) && !NTuple.Eq.equals(x, b));

  graph.removeNode(a);
  graph.removeNode(b);

  graph.addNode(newNTuple);

  for (const node of adjacentNodes) {
    if (newNTuple.intersectsWith(node)) graph.addEdge(newNTuple, node, 0);
  }
}

export function joinNodesOnEdge(graph: Graph, edge: Edge): NTuple {
  const joinedNTuple = edge[0].intersect(edge[1]);

  replaceNodes(graph, edge[0], edge[1], joinedNTuple);

  return joinedNTuple;
}

export function minimumBy<T>(list: T[], lens: (_: T) => number): T {
  return list.reduce((minimum, x) => (lens(x) < lens(minimum) ? x : minimum));
}

export function numberOfConnectedComponentsComponents(graph: Graph): number {
  const visited = new Map<string, boolean>();

  const DFSUtil = (node: NTuple) => {
    visited.set(node.id, true);

    const neighbours = graph.getNeighbours(node);
    for (const neighbour of neighbours) {
      if (!visited.get(neighbour.id)) {
        DFSUtil(neighbour);
      }
    }
  };

  let componentCount = 0;
  for (const node of graph.nodes) {
    if (!visited.get(node.id)) {
      DFSUtil(node);
      ++componentCount;
    }
  }

  return componentCount;
}

export function dfs(
  graph: Graph,
  startingNode: NTuple,
  events: {
    discoverNode?: (node: NTuple) => void;
  },
): void {
  const visited = new Map<string, boolean>();

  const DFSUtil = (node: NTuple) => {
    visited.set(node.id, true);
    events.discoverNode?.(node);

    const neighbours = graph.getNeighbours(node);
    for (const neighbour of neighbours) {
      if (!visited.get(neighbour.id)) {
        DFSUtil(neighbour);
      }
    }
  };

  DFSUtil(startingNode);
}
