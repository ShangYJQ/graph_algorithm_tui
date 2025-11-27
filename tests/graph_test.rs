use tui::graph::{EdgeType::Both, EdgeType::Single, Graph};

#[test]
fn runs_main_logic_without_panic() {
    let mut g = Graph::new();

    g.add_edge(1, 2, 10, Single);
    g.add_edge(2, 4, 10, Both);

    g.add_edge(3, 4, 10, Single);
    g.add_edge(4, 3, 10, Single);

    g.dfs(1);
    g.bfs(1);
    g.dijkstra(1);
    g.prim(1);
}
