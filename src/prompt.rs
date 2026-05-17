use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PromptKind {
    OpenFile,
    SaveAs,
}

pub struct Prompt {
    pub title: &'static str,
    pub buffer: String,
    pub kind: PromptKind,
}

pub enum PromptResult {
    Pending,
    Cancelled,
    Submitted(String),
}

impl Prompt {
    pub fn open_file() -> Self {
        Self {
            title: "Open file",
            buffer: String::new(),
            kind: PromptKind::OpenFile,
        }
    }

    pub fn save_as(initial: &str) -> Self {
        Self {
            title: "Save as",
            buffer: initial.to_string(),
            kind: PromptKind::SaveAs,
        }
    }

    pub fn on_key(&mut self, key: KeyEvent) -> PromptResult {
        match key.code {
            KeyCode::Char(c) => {
                self.buffer.push(c);
                PromptResult::Pending
            }
            KeyCode::Backspace => {
                self.buffer.pop();
                PromptResult::Pending
            }
            KeyCode::Enter => PromptResult::Submitted(std::mem::take(&mut self.buffer)),
            KeyCode::Esc => PromptResult::Cancelled,
            _ => PromptResult::Pending,
        }
    }

    pub fn render(&self, frame: &mut Frame<'_>, screen: Rect) {
        let width = 60.min(screen.width.saturating_sub(4));
        let height = 3u16;
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
                format!(" {} ", self.title),
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ));
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let line = Line::from(vec![
            Span::raw(&self.buffer),
            Span::styled(
                "▏",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        ]);
        frame.render_widget(Paragraph::new(line), inner);
    }
}
