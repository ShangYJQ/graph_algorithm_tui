use crossterm::event;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use force_graph::{DefaultNodeIdx, EdgeData, ForceGraph, NodeData, SimulationParameters};
use graph_algorithm_tui::graph::EdgeType::Both;
use graph_algorithm_tui::graph::Graph;
use graph_algorithm_tui::menu::{Menu, MenuItem, MenuSignal, MenuState};
use rand::Rng;
use ratatui::layout::{Constraint, Layout};
use ratatui::prelude::{Color, Direction};
use ratatui::style::Stylize;
use ratatui::widgets::canvas::{Canvas, Circle, Context, Line as CanvaLine};
use ratatui::widgets::{Block, Borders, Padding, Paragraph};
use ratatui::{DefaultTerminal, Frame};
use std::collections::{HashMap, HashSet};
use std::io;
use std::time::Duration;

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();

    let app_result = App::new().run(&mut terminal);

    ratatui::restore();
    app_result
}

struct App {
    data_graph: Graph,

    screen_max_x: f64,
    screen_max_y: f64,

    anchor_x: f64,
    anchor_y: f64,
    r: f64,

    dt: f64,

    anchor_idx: Option<DefaultNodeIdx>,
    graph: ForceGraph<i64, i64>,

    menu: MenuState,

    exit: bool,

    visited_nodes: HashSet<i64>,
    visited_edges: HashSet<(i64, i64)>,

    animation_nodes: Vec<i64>,
    animation_edges: Vec<(i64, i64)>,
    animation_index: usize,
    animation_timer: f64,
    animation_step_is_edge: bool,

    current_algorithm: String,
    visit_log: Vec<String>,

    prim_total_cost: i64,
    dijkstra_dist: HashMap<i64, i64>,
    dijkstra_parent: HashMap<i64, i64>,
}

impl App {
    pub fn new() -> Self {
        Self {
            data_graph: Graph::new(),

            screen_max_x: 20.0,
            screen_max_y: 10.0,

            anchor_x: 0.0,
            anchor_y: 0.0,
            r: 0.6,

            dt: 0.005,

            anchor_idx: None,
            graph: ForceGraph::new(SimulationParameters {
                force_charge: 1.0,
                force_spring: 15.0,
                force_max: 200.0,
                node_speed: 10000.0,
                damping_factor: 0.85,
            }),

            menu: MenuState::new(vec![
                MenuItem::new("遍历", vec![MenuItem::leaf("Dfs"), MenuItem::leaf("Bfs")]),
                MenuItem::new("MST", vec![MenuItem::leaf("Prim")]),
                MenuItem::new("最短路径", vec![MenuItem::leaf("Dijkstra")]),
                MenuItem::leaf("退出"),
            ]),
            exit: false,

            visited_nodes: HashSet::new(),
            visited_edges: HashSet::new(),

            animation_nodes: Vec::new(),
            animation_edges: Vec::new(),
            animation_index: 0,
            animation_timer: 0.0,
            animation_step_is_edge: false,

            current_algorithm: String::new(),
            visit_log: Vec::new(),

            prim_total_cost: 0,
            dijkstra_dist: HashMap::new(),
            dijkstra_parent: HashMap::new(),
        }
    }
    pub fn init_graph(&mut self) {
        let mut rng = rand::rng();

        let mut nodes = self.data_graph.nodes();

        if nodes.is_empty() {
            let n1_idx = self.graph.add_node(NodeData {
                x: self.anchor_x as f32,
                y: self.anchor_y as f32,
                is_anchor: true,
                user_data: 1,
                ..Default::default()
            });
            self.anchor_idx = Some(n1_idx);
            return;
        }

        nodes.sort();

        let mut id_to_idx: HashMap<i64, DefaultNodeIdx> = HashMap::new();
        let mut anchor_idx: Option<DefaultNodeIdx> = None;

        for node_id in nodes {
            let is_anchor = node_id == 1;
            let (x, y) = if is_anchor {
                (self.anchor_x as f32, self.anchor_y as f32)
            } else {
                (rng.random_range(-1.0..1.0), rng.random_range(-1.0..1.0))
            };

            let idx = self.graph.add_node(NodeData {
                x,
                y,
                is_anchor,
                user_data: node_id,
                ..Default::default()
            });

            if is_anchor {
                anchor_idx = Some(idx);
            }

            id_to_idx.insert(node_id, idx);
        }

        self.anchor_idx = anchor_idx;

        for (u, v, w) in self.data_graph.edges() {
            if let (Some(&u_idx), Some(&v_idx)) = (id_to_idx.get(&u), id_to_idx.get(&v)) {
                self.graph.add_edge(u_idx, v_idx, EdgeData { user_data: w });
            }
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        self.data_graph.add_edge(1, 2, 2, Both);
        self.data_graph.add_edge(1, 3, 3, Both);
        self.data_graph.add_edge(3, 4, 5, Both);
        self.data_graph.add_edge(3, 5, 4, Both);
        self.data_graph.add_edge(5, 4, 1, Both);
        self.data_graph.add_edge(1, 6, 6, Both);

        self.init_graph();
        while !self.exit {
            self.handle_events()?;

            self.update_animation();

            let limit_x = (self.screen_max_x - self.r) as f32;
            let limit_y = (self.screen_max_y - self.r) as f32;
            if let Some(idx) = self.anchor_idx {
                let tx = self.anchor_x as f32;
                let ty = self.anchor_y as f32;

                self.graph.visit_nodes_mut(|node| {
                    if node.index() == idx {
                        node.data.x = tx;
                        node.data.y = ty;
                    } else {
                        node.data.x = node.data.x.clamp(-limit_x, limit_x);
                        node.data.y = node.data.y.clamp(-limit_y, limit_y);
                    }
                });
            }

            self.graph.update(self.dt as f32);
            terminal.draw(|frame| self.draw(frame))?;
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(80), Constraint::Percentage(20)])
            .split(frame.area());

        let canva = Canvas::default()
            .block(Block::default().title("Graph").borders(Borders::ALL))
            .x_bounds([-self.screen_max_x, self.screen_max_x])
            .y_bounds([-self.screen_max_y, self.screen_max_y])
            .paint(|ctx| self.render_ctx(ctx));

        frame.render_widget(canva, chunks[0]);

        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
            .split(chunks[1]);

        let title = "Menu";

        let menu_widget = Menu::new()
            .block(Block::default().title(title).borders(Borders::ALL))
            .highlight_style(
                ratatui::style::Style::default()
                    .bg(Color::Blue)
                    .fg(Color::White),
            ); // 设置高亮样式

        frame.render_stateful_widget(menu_widget, right_chunks[0], &mut self.menu);

        let mut log_lines = self.visit_log.clone();

        let animation_complete = self.animation_index >= self.animation_nodes.len();

        if animation_complete && !self.current_algorithm.is_empty() {
            log_lines.push("".to_string());
            log_lines.push("---- 结束 ----\n".to_string());

            match self.current_algorithm.as_str() {
                "Prim" => {
                    log_lines.push(format!("最小生成树总长度: {}", self.prim_total_cost));
                }
                "Dijkstra" => {
                    log_lines.push("最短距离:".to_string());
                    let mut sorted_nodes: Vec<_> = self.dijkstra_dist.iter().collect();
                    sorted_nodes.sort_by_key(|(k, _)| *k);

                    for (&node, &dist) in sorted_nodes {
                        // Build path
                        let mut path = vec![node];
                        let mut current = node;
                        while let Some(&prev) = self.dijkstra_parent.get(&current) {
                            path.push(prev);
                            current = prev;
                        }
                        path.reverse();

                        let path_str = path
                            .iter()
                            .map(|n| n.to_string())
                            .collect::<Vec<_>>()
                            .join(" -> ");

                        log_lines.push(format!(
                            "  到节点{}: 距离={}, 路径={}",
                            node, dist, path_str
                        ));
                    }
                }
                _ => {}
            }
        }

        let log_text = log_lines.join("\n");
        let info_title = if self.current_algorithm.is_empty() {
            "请选择算法".to_string()
        } else {
            format!("{}", self.current_algorithm)
        };

        let info_widget = Paragraph::new(log_text).block(
            Block::default()
                .title(info_title)
                .borders(Borders::ALL)
                .padding(Padding::uniform(1)),
        );

        frame.render_widget(info_widget, right_chunks[1]);
    }

    fn render_ctx(&self, ctx: &mut Context) {
        self.graph.visit_edges(|node1, node2, edge_data| {
            let u = node1.data.user_data;
            let v = node2.data.user_data;

            let is_visited =
                self.visited_edges.contains(&(u, v)) || self.visited_edges.contains(&(v, u));

            let x1 = node1.x() as f64;
            let y1 = node1.y() as f64;
            let x2 = node2.x() as f64;
            let y2 = node2.y() as f64;

            ctx.draw(&CanvaLine {
                x1,
                y1,
                x2,
                y2,
                color: if is_visited {
                    Color::Yellow
                } else {
                    Color::LightBlue
                },
            });

            let mid_x = (x1 + x2) / 2.0;
            let mid_y = (y1 + y2) / 2.0;
            ctx.print(mid_x, mid_y, format!("{}", edge_data.user_data).white());
        });

        self.graph.visit_nodes(|node| {
            let node_id = node.data.user_data;
            let is_visited = self.visited_nodes.contains(&node_id);

            ctx.draw(&Circle {
                x: node.x() as f64,
                y: node.y() as f64,
                radius: self.r,
                color: if is_visited {
                    Color::Yellow
                } else {
                    Color::LightBlue
                },
            });
            ctx.print(
                node.x() as f64,
                node.y() as f64,
                format!("{}", node.data.user_data).yellow(),
            );
        });
    }

    fn update_animation(&mut self) {
        if self.animation_nodes.is_empty() {
            return;
        }

        let nodes_done = self.animation_index >= self.animation_nodes.len();
        let edges_done = self.animation_index >= self.animation_edges.len();

        if nodes_done && (edges_done || !self.animation_step_is_edge) {
            return;
        }

        self.animation_timer += self.dt;

        if self.animation_timer >= 0.2 {
            self.animation_timer = 0.0;

            if self.animation_step_is_edge {
                if self.animation_index < self.animation_edges.len() {
                    let edge = self.animation_edges[self.animation_index];
                    self.visited_edges.insert(edge);
                    self.visit_log
                        .push(format!("访问边: {} -> {}", edge.0, edge.1));
                }
                self.animation_index += 1;
                self.animation_step_is_edge = false;
            } else {
                if self.animation_index < self.animation_nodes.len() {
                    let node = self.animation_nodes[self.animation_index];
                    self.visited_nodes.insert(node);
                    self.visit_log.push(format!("访问节点: {}", node));
                }
                if self.animation_index < self.animation_edges.len() {
                    self.animation_step_is_edge = true;
                } else {
                    self.animation_index += 1;
                }
            }
        }
    }

    fn run_dfs(&mut self) {
        self.current_algorithm = "DFS".to_string();
        self.visit_log.clear();

        self.visited_nodes.clear();
        self.visited_edges.clear();

        let (nodes, edges) = self.data_graph.dfs(1);
        self.animation_nodes = nodes;
        self.animation_edges = edges;

        if !self.animation_nodes.is_empty() {
            self.visited_nodes.insert(self.animation_nodes[0]);
            self.visit_log
                .push(format!("访问节点: {}", self.animation_nodes[0]));
        }

        self.animation_index = 0;
        self.animation_timer = 0.0;
        self.animation_step_is_edge = true;
    }

    fn run_bfs(&mut self) {
        // Set algorithm name and clear log
        self.current_algorithm = "BFS".to_string();
        self.visit_log.clear();

        self.visited_nodes.clear();
        self.visited_edges.clear();

        let (nodes, edges) = self.data_graph.bfs(1);
        self.animation_nodes = nodes;
        self.animation_edges = edges;

        if !self.animation_nodes.is_empty() {
            self.visited_nodes.insert(self.animation_nodes[0]);
            self.visit_log
                .push(format!("访问节点: {}", self.animation_nodes[0]));
        }

        self.animation_index = 0;
        self.animation_timer = 0.0;
        self.animation_step_is_edge = true;
    }

    fn run_prim(&mut self) {
        self.current_algorithm = "Prim".to_string();
        self.visit_log.clear();

        self.visited_nodes.clear();
        self.visited_edges.clear();

        let (nodes, edges, total_cost) = self.data_graph.prim(1);
        self.animation_nodes = nodes;
        self.animation_edges = edges;
        self.prim_total_cost = total_cost;

        if !self.animation_nodes.is_empty() {
            self.visited_nodes.insert(self.animation_nodes[0]);
            self.visit_log
                .push(format!("访问节点: {}", self.animation_nodes[0]));
        }

        self.animation_index = 0;
        self.animation_timer = 0.0;
        self.animation_step_is_edge = true;
    }

    fn run_dijkstra(&mut self) {
        self.current_algorithm = "Dijkstra".to_string();
        self.visit_log.clear();

        self.visited_nodes.clear();
        self.visited_edges.clear();

        let (nodes, edges, dist, parent) = self.data_graph.dijkstra(1);
        self.animation_nodes = nodes;
        self.animation_edges = edges;
        self.dijkstra_dist = dist;
        self.dijkstra_parent = parent;

        if !self.animation_nodes.is_empty() {
            self.visited_nodes.insert(self.animation_nodes[0]);
            self.visit_log
                .push(format!("访问节点: {}", self.animation_nodes[0]));
        }

        self.animation_index = 0;
        self.animation_timer = 0.0;
        self.animation_step_is_edge = true;
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::poll(Duration::from_secs_f32(self.dt as f32))? {
            true => match event::read()? {
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    let limit_x = self.screen_max_x - self.r;
                    let limit_y = self.screen_max_y - self.r;

                    match key_event.code {
                        KeyCode::Right => {
                            self.anchor_x = (self.anchor_x + 0.2).clamp(-limit_x, limit_x)
                        }
                        KeyCode::Up => {
                            self.anchor_y = (self.anchor_y + 0.2).clamp(-limit_y, limit_y)
                        }
                        KeyCode::Down => {
                            self.anchor_y = (self.anchor_y - 0.2).clamp(-limit_y, limit_y)
                        }
                        KeyCode::Left => {
                            self.anchor_x = (self.anchor_x - 0.2).clamp(-limit_x, limit_x)
                        }

                        KeyCode::Char('+') => self.r += 0.1,
                        KeyCode::Char('-') => self.r -= 0.1,

                        // menu
                        KeyCode::Char('j') => self.menu.down(),
                        KeyCode::Char('k') => self.menu.up(),
                        KeyCode::Char('l') | KeyCode::Enter => match self.menu.enter() {
                            MenuSignal::Selected(name) => match name.as_str() {
                                "Bfs" => self.run_bfs(),
                                "Dfs" => self.run_dfs(),
                                "Prim" => self.run_prim(),
                                "Dijkstra" => self.run_dijkstra(),
                                "退出" => self.exit = true,
                                _ => {}
                            },
                            MenuSignal::None => {}
                        },
                        KeyCode::Char('h') => self.menu.back(),

                        KeyCode::Char('q') => self.exit = true,
                        _ => {}
                    }
                }
                _ => {}
            },
            false => (),
        }

        Ok(())
    }
}
