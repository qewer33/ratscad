use std::sync::mpsc::{Receiver, TryRecvError};

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Gauge, Paragraph};

use crate::openscad::InstallMsg;

pub enum InstallStatus {
    Starting,
    Downloading { downloaded: u64, total: Option<u64> },
    Failed(String),
}

pub struct InstallPopup {
    rx: Receiver<InstallMsg>,
    url: &'static str,
    status: InstallStatus,
}

pub enum InstallOutcome {
    InProgress,
    Done(std::path::PathBuf),
    Failed(String),
}

impl InstallPopup {
    pub fn new(rx: Receiver<InstallMsg>, url: &'static str) -> Self {
        Self {
            rx,
            url,
            status: InstallStatus::Starting,
        }
    }

    // Drains messages from the install worker. Returns Done with the final
    // path if the install finished, Failed on error, or InProgress otherwise.
    pub fn poll(&mut self) -> InstallOutcome {
        loop {
            match self.rx.try_recv() {
                Ok(InstallMsg::Progress { downloaded, total }) => {
                    self.status = InstallStatus::Downloading { downloaded, total };
                }
                Ok(InstallMsg::Done(path)) => return InstallOutcome::Done(path),
                Ok(InstallMsg::Failed(err)) => {
                    self.status = InstallStatus::Failed(err.clone());
                    return InstallOutcome::Failed(err);
                }
                Err(TryRecvError::Empty) => return InstallOutcome::InProgress,
                Err(TryRecvError::Disconnected) => {
                    return InstallOutcome::Failed("install worker died".to_string());
                }
            }
        }
    }

    pub fn render(&self, frame: &mut Frame<'_>, screen: Rect) {
        let width = 70u16.min(screen.width.saturating_sub(4)).max(30);
        let height = 8u16.min(screen.height.saturating_sub(2)).max(6);
        let x = screen.x + screen.width.saturating_sub(width) / 2;
        let y = screen.y + screen.height.saturating_sub(height) / 2;
        let area = Rect {
            x,
            y,
            width,
            height,
        };

        frame.render_widget(Clear, area);
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow))
            .title(Span::styled(
                " Installing OpenSCAD ",
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ));
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(inner);

        // Line 1: blank padding row, then URL on line 2.
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled("from ", Style::default().fg(Color::DarkGray)),
                Span::styled(self.url, Style::default().fg(Color::Gray)),
            ])),
            chunks[1],
        );

        match &self.status {
            InstallStatus::Starting => {
                frame.render_widget(
                    Paragraph::new(Line::from(vec![Span::styled(
                        "preparing...",
                        Style::default().fg(Color::Gray),
                    )])),
                    chunks[2],
                );
            }
            InstallStatus::Downloading { downloaded, total } => {
                let ratio = match total {
                    Some(t) if *t > 0 => {
                        (*downloaded as f64 / *t as f64).clamp(0.0, 1.0)
                    }
                    _ => 0.0,
                };
                let label = match total {
                    Some(t) => format!("{} / {}", format_bytes(*downloaded), format_bytes(*t)),
                    None => format_bytes(*downloaded),
                };
                let gauge = Gauge::default()
                    .gauge_style(
                        Style::default()
                            .fg(Color::Yellow)
                            .bg(Color::Indexed(236))
                            .add_modifier(Modifier::BOLD),
                    )
                    .ratio(ratio)
                    .label(label);
                frame.render_widget(gauge, chunks[2]);
            }
            InstallStatus::Failed(err) => {
                frame.render_widget(
                    Paragraph::new(Line::from(vec![
                        Span::styled("failed: ", Style::default().fg(Color::Red)),
                        Span::styled(err.as_str(), Style::default().fg(Color::Red)),
                    ])),
                    chunks[2],
                );
            }
        }

        frame.render_widget(
            Paragraph::new(Line::from(vec![Span::styled(
                "press Ctrl+Q to abort",
                Style::default().fg(Color::DarkGray),
            )])),
            chunks[3],
        );
    }
}

fn format_bytes(n: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    if n >= GB {
        format!("{:.2} GB", n as f64 / GB as f64)
    } else if n >= MB {
        format!("{:.1} MB", n as f64 / MB as f64)
    } else if n >= KB {
        format!("{:.1} KB", n as f64 / KB as f64)
    } else {
        format!("{} B", n)
    }
}
