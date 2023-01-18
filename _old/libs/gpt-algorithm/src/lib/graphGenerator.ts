import { Graph, NTuple, type IInput } from '@testing-repo/gpt-common';
import { generateTestCases } from './testCaseGenerator';

export function generateGraph(inputs: IInput[][]): Graph {
  const nTuples = inputs.flatMap(generateTestCases);

  const graph = new Graph();
  graph.addNodes(nTuples);

  for (const item1 of nTuples) {
    for (const item2 of nTuples) {
      if (NTuple.Eq.equals(item1, item2)) continue;

      if (item1.intersectsWith(item2)) graph.addEdge(item1, item2, Infinity);
    }
  }

  return graph;
}

export function createGraphUrl(graph: Graph): string {
  const mapping = new Map<string, number>(
    graph.nodes.map((x, index) => [x.id, index]),
  );
  const getId = (node: NTuple) => mapping.get(node.id);

  const nodes = graph.nodes
    .map((node) => `${getId(node)} [label="${node}"]`)
    .join('\n');
  const edges = graph.edges
    .map(
      ([from, to, weight]) =>
        `${getId(from)} -- ${getId(to)} [label="${weight}"]`,
    )
    .join('\n')
    .replaceAll('Infinity', '∞');

  const dot = `graph G {
    node [shape=box]
    ${nodes}
    ${edges}
  }`;

  return encodeURI(`https://dreampuf.github.io/GraphvizOnline/#${dot}`);
}
