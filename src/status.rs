use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::app::Focus;

pub enum BuildStatus {
    Idle,
    Building,
    Ready { bytes: usize },
    Failed(String),
}

pub struct GlobalToolbar<'a> {
    pub status: &'a BuildStatus,
    pub focus: Focus,
}

impl<'a> GlobalToolbar<'a> {
    pub fn render(&self, frame: &mut Frame<'_>, area: Rect) {
        let split = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(1), Constraint::Length(40)])
            .split(area);
        self.render_shortcuts(frame, split[0]);
        self.render_state(frame, split[1]);
    }

    fn render_shortcuts(&self, frame: &mut Frame<'_>, area: Rect) {
        let bar_bg = Color::Indexed(236);
        let key = Style::default()
            .fg(Color::Black)
            .bg(Color::Yellow)
            .add_modifier(Modifier::BOLD);
        let desc = Style::default().fg(Color::Gray).bg(bar_bg);
        let sep = Style::default().fg(Color::DarkGray).bg(bar_bg);
        let spans = vec![
            Span::styled(" ", desc),
            Span::styled("Ctrl", sep),
            Span::styled(" + ", sep),
            Span::styled(" q ", key),
            Span::styled(" quit ", desc),
            Span::styled(" s ", key),
            Span::styled(" save ", desc),
            Span::styled(" o ", key),
            Span::styled(" open ", desc),
            Span::styled(" v ", key),
            Span::styled(" viewer ", desc),
            Span::styled(" e ", key),
            Span::styled(" editor", desc),
            Span::styled("    ", desc),
            Span::styled("Alt", sep),
            Span::styled(" + ", sep),
            Span::styled(" m ", key),
            Span::styled(" menu", desc),
        ];
        frame.render_widget(
            Paragraph::new(Line::from(spans)).style(Style::default().bg(bar_bg)),
            area,
        );
    }

    fn render_state(&self, frame: &mut Frame<'_>, area: Rect) {
        let bar_bg = Color::Indexed(236);
        let (label, color) = match self.status {
            BuildStatus::Idle => ("idle".to_string(), Color::DarkGray),
            BuildStatus::Building => ("building...".to_string(), Color::Yellow),
            BuildStatus::Ready { bytes } => (format!("ready ({bytes}b)"), Color::Green),
            BuildStatus::Failed(msg) => (format!("failed: {}", first_error_line(msg)), Color::Red),
        };
        let (focus_label, focus_color) = match self.focus {
            Focus::Editor => ("EDITOR", Color::Cyan),
            Focus::Viewer => ("VIEWER", Color::Magenta),
            Focus::Menubar => ("MENU", Color::Green),
        };
        let line = Line::from(vec![
            Span::styled(label.as_str(), Style::default().fg(color).bg(bar_bg)),
            Span::styled("  ", Style::default().bg(bar_bg)),
            Span::styled(
                format!(" {focus_label} "),
                Style::default()
                    .fg(Color::Black)
                    .bg(focus_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" ", Style::default().bg(bar_bg)),
        ])
        .right_aligned();
        frame.render_widget(
            Paragraph::new(line).style(Style::default().bg(bar_bg)),
            area,
        );
    }
}

pub fn key_style() -> Style {
    Style::default()
        .fg(Color::Black)
        .bg(Color::Yellow)
        .add_modifier(Modifier::BOLD)
}

pub fn desc_style() -> Style {
    Style::default().fg(Color::Gray)
}

pub fn sep_style() -> Style {
    Style::default().fg(Color::DarkGray)
}

fn first_error_line(stderr: &str) -> &str {
    stderr
        .lines()
        .find(|l| l.contains("ERROR"))
        .unwrap_or_else(|| stderr.lines().next().unwrap_or(""))
}
