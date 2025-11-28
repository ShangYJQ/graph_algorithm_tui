use crossterm::event;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use force_graph::{DefaultNodeIdx, ForceGraph, NodeData, SimulationParameters};
use graph_algorithm_tui::graph::EdgeType::Both;
use graph_algorithm_tui::graph::Graph;
use rand::Rng;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::prelude::{Color, Direction};
use ratatui::style::Stylize;
use ratatui::widgets::canvas::{Canvas, Circle, Context, Line as CanvaLine};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::{DefaultTerminal, Frame};
use std::collections::HashMap;
use std::io;
use std::time::Duration;

use graph_algorithm_tui::menu::{Menu, MenuItem, MenuSignal, MenuState};

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

        for (u, v, _w) in self.data_graph.edges() {
            if let (Some(&u_idx), Some(&v_idx)) = (id_to_idx.get(&u), id_to_idx.get(&v)) {
                self.graph.add_edge(u_idx, v_idx, Default::default());
            }
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        self.data_graph.add_edge(1, 2, 1, Both);
        self.data_graph.add_edge(1, 3, 1, Both);
        self.data_graph.add_edge(3, 4, 1, Both);
        self.data_graph.add_edge(3, 5, 1, Both);
        self.data_graph.add_edge(5, 4, 1, Both);
        self.data_graph.add_edge(1, 6, 1, Both);

        self.init_graph();
        while !self.exit {
            self.handle_events()?;

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

        let title = "Menu";

        let menu_widget = Menu::new()
            .block(Block::default().title(title).borders(Borders::ALL))
            .highlight_style(
                ratatui::style::Style::default()
                    .bg(Color::Blue)
                    .fg(Color::White),
            ); // 设置高亮样式

        frame.render_stateful_widget(menu_widget, chunks[1], &mut self.menu);
    }

    fn render_ctx(&self, ctx: &mut Context) {
        self.graph.visit_edges(|node1, node2, _edge_data| {
            ctx.draw(&CanvaLine {
                x1: node1.x() as f64,
                y1: node1.y() as f64,
                x2: node2.x() as f64,
                y2: node2.y() as f64,
                color: Color::Blue,
            });
        });

        self.graph.visit_nodes(|node| {
            ctx.draw(&Circle {
                x: node.x() as f64,
                y: node.y() as f64,
                radius: self.r,
                color: Color::Green,
            });
            ctx.print(
                node.x() as f64,
                node.y() as f64,
                format!("{}", node.data.user_data).yellow(),
            );
        });
    }

    fn run_dfs(&self) {
        todo!()
    }

    fn run_bfs(&self) {
        todo!()
    }
    fn run_prim(&self) {
        todo!()
    }
    fn run_dijkstra(&self) {
        todo!()
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

// impl ratatui::prelude::Widget for &App {
//     fn render(self, area: Rect, buf: &mut Buffer) {
//         let chunks = Layout::default()
//             .direction(Direction::Horizontal)
//             .constraints([Constraint::Percentage(80), Constraint::Percentage(20)])
//             .split(area);
//
//         let canva = Canvas::default()
//             .block(Block::default().title("Graph").borders(Borders::ALL))
//             .x_bounds([-self.screen_max_x, self.screen_max_x])
//             .y_bounds([-self.screen_max_y, self.screen_max_y])
//             .paint(|ctx| self.render_ctx(ctx));
//
//         canva.render(chunks[0], buf);
//
//         Paragraph::new("")
//             .block(Block::default().title("Menu").borders(Borders::ALL))
//             .render(chunks[1], buf);
//     }
// }
