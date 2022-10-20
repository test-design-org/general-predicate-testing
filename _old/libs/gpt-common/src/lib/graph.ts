import { NTuple } from './ntuple';

export type Edge = [NTuple, NTuple, number];

export class Graph {
  nodes: NTuple[];
  edges: Edge[];

  constructor(nodes: NTuple[] = [], edges: Edge[] = []) {
    this.nodes = nodes;
    this.edges = edges;
  }

  addNode(node: NTuple) {
    // If it is already in the graph, don't add it
    if (this.nodes.some((x) => NTuple.Eq.equals(x, node))) return;

    this.nodes.push(node);
  }

  addNodes(nodes: NTuple[]) {
    nodes.forEach((x) => this.addNode(x));
  }

  hasEdge(from: NTuple, to: NTuple): boolean {
    return this.edges.some((edge) => this.isEdge(from, to, edge));
  }

  private isEdge(from: NTuple, to: NTuple, edge: Edge): boolean {
    const [x, y] = edge;
    const containsThisWay =
      NTuple.Eq.equals(x, from) && NTuple.Eq.equals(y, to);
    const containsThatWay =
      NTuple.Eq.equals(y, from) && NTuple.Eq.equals(x, to);

    return containsThisWay || containsThatWay;
  }

  addEdge(from: NTuple, to: NTuple, weight: number) {
    if (this.hasEdge(from, to)) return;

    this.edges.push([from, to, weight]);
  }

  shallowClone(): Graph {
    const edgesClone = this.edges.map(([x, y, z]) => [x, y, z] as Edge);
    return new Graph(this.nodes, edgesClone);
  }

  getNeighbours(node: NTuple): NTuple[] {
    const neighbours = this.edges
      .map(([from, to, _]) => {
        if (NTuple.Eq.equals(from, node)) return to;

        if (NTuple.Eq.equals(to, node)) return from;

        return null;
      })
      .filter((x) => x !== null);

    return neighbours as NTuple[];
  }

  removeNode(node: NTuple) {
    this.nodes = this.nodes.filter((x) => !NTuple.Eq.equals(x, node));
    this.edges = this.edges.filter(
      ([x, y, _]) => !NTuple.Eq.equals(x, node) && !NTuple.Eq.equals(y, node),
    );
  }

  getNode(id: string) {
    const node = this.nodes.find((x) => x.id === id);
    if (node === undefined)
      throw new Error(`Node with id ${id} not found in graph`);

    return node;
  }
}
