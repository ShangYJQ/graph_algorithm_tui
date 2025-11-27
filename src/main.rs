use color_eyre::owo_colors::OwoColorize;
use crossterm::event;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use force_graph::{DefaultNodeIdx, ForceGraph, NodeData, SimulationParameters};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::{Color, Widget};
use ratatui::style::Stylize;
use ratatui::symbols::border;
use ratatui::text::{Line, Text};
use ratatui::widgets::canvas::{Canvas, Circle, Context, Line as CanvaLine};
use ratatui::widgets::{Block, Paragraph};
use ratatui::{DefaultTerminal, Frame};
use std::time::Duration;
use std::{io, thread};

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();

    let app_result = App::new().run(&mut terminal);

    ratatui::restore();
    app_result
}

struct App {
    screen_max_x: f64,
    screen_max_y: f64,

    anchor_x: f64,
    anchor_y: f64,
    r: f64,

    dt: f64,

    anchor_idx: Option<DefaultNodeIdx>,
    graph: ForceGraph<i64, i64>,

    exit: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            screen_max_x: 20.0,
            screen_max_y: 10.0,

            anchor_x: 0.0,
            anchor_y: 0.0,
            r: 0.6,

            dt: 0.005,

            anchor_idx: None,
            graph: ForceGraph::new(SimulationParameters {
                force_charge: 0.7,
                force_spring: 15.0,
                force_max: 200.0,
                node_speed: 10000.0,
                damping_factor: 0.8,
            }),

            exit: false,
        }
    }

    pub fn init_graph(&mut self) {
        let n1_idx = self.graph.add_node(NodeData {
            x: self.anchor_x as f32,
            y: self.anchor_y as f32,
            is_anchor: true,
            user_data: 1,
            ..Default::default()
        });

        self.anchor_idx = Some(n1_idx);

        let n2_idx = self.graph.add_node(NodeData {
            x: 0.0,
            y: 0.0,
            user_data: 2,

            ..Default::default()
        });
        let n3_idx = self.graph.add_node(NodeData {
            x: -0.0,
            y: 0.0,
            user_data: 3,

            ..Default::default()
        });
        let n4_idx = self.graph.add_node(NodeData {
            x: -0.0,
            y: -0.1,

            user_data: 4,

            ..Default::default()
        });
        let n5_idx = self.graph.add_node(NodeData {
            x: -0.0,
            y: -0.0,

            user_data: 5,

            ..Default::default()
        });

        self.graph.add_edge(n1_idx, n3_idx, Default::default());
        self.graph.add_edge(n1_idx, n2_idx, Default::default());
        self.graph.add_edge(n2_idx, n5_idx, Default::default());

        self.graph.add_edge(n2_idx, n4_idx, Default::default());
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
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
            thread::sleep(Duration::from_secs_f64(self.dt));
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
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

    fn handle_events(&mut self) -> io::Result<()> {
        if (event::poll(Duration::from_secs_f64(self.dt))?) {
            match event::read()? {
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
                        KeyCode::Esc => self.exit = true,
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let canva = Canvas::default()
            .block(
                Block::default()
                    .title("Canvas")
                    .borders(ratatui::widgets::Borders::ALL),
            )
            .x_bounds([-self.screen_max_x, self.screen_max_x])
            .y_bounds([-self.screen_max_y, self.screen_max_y])
            .paint(|ctx| self.render_ctx(ctx));

        canva.render(area, buf);
    }
}
