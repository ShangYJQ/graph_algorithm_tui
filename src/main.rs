use crate::graph::Graph;

mod graph;

fn main() {

    let mut g = Graph::new();

    g.add_edge(1,2,2);
    g.add_edge(1,3,2);
    g.add_edge(1,4,3);
    g.add_edge(2,3,4);
    g.add_edge(3,4,3);

    g.dfs(1);
    g.bfs(1);
    g.dijkstra(1);
    g.prim(1);

}