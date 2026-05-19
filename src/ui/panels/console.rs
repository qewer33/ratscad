use std::collections::VecDeque;

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

const MAX_ENTRIES: usize = 200;

#[derive(Clone, Copy)]
pub enum ConsoleLevel {
    Building,
    Success,
    Error,
}

struct ConsoleEntry {
    level: ConsoleLevel,
    text: String,
}

pub struct ConsolePane {
    entries: VecDeque<ConsoleEntry>,
}

impl ConsolePane {
    pub fn new() -> Self {
        Self {
            entries: VecDeque::new(),
        }
    }

    pub fn push(&mut self, level: ConsoleLevel, text: impl Into<String>) {
        self.entries.push_back(ConsoleEntry {
            level,
            text: text.into(),
        });
        while self.entries.len() > MAX_ENTRIES {
            self.entries.pop_front();
        }
    }

    pub fn render(&self, frame: &mut Frame<'_>, area: Rect) {
        let block = Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(Span::styled(
                " output ",
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ));
        let inner = block.inner(area);
        ratatui::widgets::Widget::render(block, area, frame.buffer_mut());

        if inner.height == 0 {
            return;
        }

        let lines: Vec<Line> = self
            .entries
            .iter()
            .flat_map(|entry| {
                let (color, prefix) = match entry.level {
                    ConsoleLevel::Building => (Color::Yellow, "» "),
                    ConsoleLevel::Success => (Color::Green, "✓ "),
                    ConsoleLevel::Error => (Color::Red, "✗ "),
                };
                let style = Style::default().fg(color);
                entry
                    .text
                    .lines()
                    .enumerate()
                    .map(move |(i, line_text)| {
                        let head = if i == 0 { prefix } else { "  " };
                        Line::from(vec![
                            Span::styled(head, style),
                            Span::styled(line_text.to_string(), style),
                        ])
                    })
                    .collect::<Vec<_>>()
            })
            .collect();

        let max_visible = inner.height as usize;
        let start = lines.len().saturating_sub(max_visible);
        let visible: Vec<Line> = lines.into_iter().skip(start).collect();

        frame.render_widget(
            Paragraph::new(visible).wrap(Wrap { trim: false }),
            inner,
        );
    }
}
