import { Graph, NTuple } from '@testing-repo/gpt-common';
import { dfs, joinNodesOnEdge, minimumBy } from './common';
import { createGraphUrl } from '../graphGenerator';

function nodesReachable(graph: Graph, startNode: NTuple): number {
  let nodeCount = 0;

  dfs(graph, startNode, {
    discoverNode: (_) => ++nodeCount,
  });

  return nodeCount;
}

function evaluateEdgesreachableCount(graph: Graph) {
  for (const edge of graph.edges) {
    const workingGraph = graph.shallowClone();

    const initiallyReachable = nodesReachable(workingGraph, edge[0]);
    const joinedNTuple = joinNodesOnEdge(workingGraph, edge);
    const reachableAfterTheJoin = nodesReachable(workingGraph, joinedNTuple);

    // -1, because we shouldn't count the starting node itself
    edge[2] = initiallyReachable - reachableAfterTheJoin - 1;
  }
  console.log(
    'Partial data, least losing nodes reachable graph: ',
    createGraphUrl(graph),
  );
}

export function runLeastLosingNodesReachable(_graph: Graph): Graph {
  const graph = _graph.shallowClone();

  while (graph.edges.length > 0) {
    evaluateEdgesreachableCount(graph);

    const edgeToJoin = minimumBy(graph.edges, (x) => x[2]);

    joinNodesOnEdge(graph, edgeToJoin);
  }

  return graph;
}
