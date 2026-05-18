use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MenuAction {
    New,
    Open,
    Save,
    SaveAs,
    Close,
    Quit,
    ToggleConsole,
}

pub struct MenuItem {
    pub label: &'static str,
    pub shortcut: &'static str,
    pub action: MenuAction,
}

pub const VIEW_MENU: &[MenuItem] = &[MenuItem {
    label: "Toggle Console",
    shortcut: "",
    action: MenuAction::ToggleConsole,
}];

pub const FILE_MENU: &[MenuItem] = &[
    MenuItem {
        label: "New",
        shortcut: "Ctrl+T",
        action: MenuAction::New,
    },
    MenuItem {
        label: "Open",
        shortcut: "Ctrl+O",
        action: MenuAction::Open,
    },
    MenuItem {
        label: "Save",
        shortcut: "Ctrl+S",
        action: MenuAction::Save,
    },
    MenuItem {
        label: "Save As",
        shortcut: "Ctrl+Shift+S",
        action: MenuAction::SaveAs,
    },
    MenuItem {
        label: "Close",
        shortcut: "Ctrl+W",
        action: MenuAction::Close,
    },
    MenuItem {
        label: "Quit",
        shortcut: "Ctrl+Q",
        action: MenuAction::Quit,
    },
];

pub struct MenuPopup {
    pub items: &'static [MenuItem],
    pub selected: usize,
    pub anchor_x: u16,
    pub anchor_y: u16,
}

pub enum MenuResult {
    Pending,
    Cancelled,
    Activated(MenuAction),
}

impl MenuPopup {
    pub fn file_menu(anchor_x: u16, anchor_y: u16) -> Self {
        Self {
            items: FILE_MENU,
            selected: 0,
            anchor_x,
            anchor_y,
        }
    }

    pub fn view_menu(anchor_x: u16, anchor_y: u16) -> Self {
        Self {
            items: VIEW_MENU,
            selected: 0,
            anchor_x,
            anchor_y,
        }
    }

    pub fn on_key(&mut self, key: KeyEvent) -> MenuResult {
        match key.code {
            KeyCode::Up => {
                self.selected = (self.selected + self.items.len() - 1) % self.items.len();
                MenuResult::Pending
            }
            KeyCode::Down => {
                self.selected = (self.selected + 1) % self.items.len();
                MenuResult::Pending
            }
            KeyCode::Enter => MenuResult::Activated(self.items[self.selected].action),
            KeyCode::Esc => MenuResult::Cancelled,
            _ => MenuResult::Pending,
        }
    }

    pub fn on_mouse(&mut self, event: MouseEvent, screen: Rect) -> MenuResult {
        let area = self.area(screen);
        let inside = event.column >= area.x
            && event.column < area.x + area.width
            && event.row >= area.y
            && event.row < area.y + area.height;

        match event.kind {
            MouseEventKind::Moved if inside => {
                if let Some(idx) = self.item_at_row(area, event.row) {
                    self.selected = idx;
                }
                MenuResult::Pending
            }
            MouseEventKind::Down(MouseButton::Left) => {
                if !inside {
                    return MenuResult::Cancelled;
                }
                if let Some(idx) = self.item_at_row(area, event.row) {
                    MenuResult::Activated(self.items[idx].action)
                } else {
                    MenuResult::Pending
                }
            }
            _ => MenuResult::Pending,
        }
    }

    fn area(&self, screen: Rect) -> Rect {
        let max_label = self.items.iter().map(|i| i.label.len()).max().unwrap_or(0);
        let max_shortcut = self.items.iter().map(|i| i.shortcut.len()).max().unwrap_or(0);
        let inner_width = max_label + max_shortcut + 4;
        let width = (inner_width as u16).saturating_add(2);
        let height = (self.items.len() as u16).saturating_add(2);
        let x = self.anchor_x.min(screen.x + screen.width.saturating_sub(width));
        let y = self.anchor_y;
        Rect {
            x,
            y,
            width,
            height,
        }
    }

    fn item_at_row(&self, area: Rect, row: u16) -> Option<usize> {
        let first_item_row = area.y + 1;
        let last_item_row = area.y + area.height - 1;
        if row >= first_item_row && row < last_item_row {
            Some((row - first_item_row) as usize)
        } else {
            None
        }
    }

    pub fn render(&self, frame: &mut Frame<'_>, screen: Rect) {
        let max_label = self.items.iter().map(|i| i.label.len()).max().unwrap_or(0);
        let max_shortcut = self.items.iter().map(|i| i.shortcut.len()).max().unwrap_or(0);
        let inner_width = max_label + max_shortcut + 4;
        let width = (inner_width as u16).saturating_add(2);
        let height = (self.items.len() as u16).saturating_add(2);
        let x = self.anchor_x.min(screen.x + screen.width.saturating_sub(width));
        let y = self.anchor_y;
        let area = Rect {
            x,
            y,
            width,
            height,
        };

        frame.render_widget(Clear, area);
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow));
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let lines: Vec<Line> = self
            .items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let active = i == self.selected;
                let item_style = if active {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Gray)
                };
                let shortcut_style = if active {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::DarkGray)
                };
                let label_padding = max_label - item.label.len();
                let mid_gap = label_padding + 2;
                Line::from(vec![
                    Span::styled(format!(" {} ", item.label), item_style),
                    Span::styled(" ".repeat(mid_gap), item_style),
                    Span::styled(format!("{} ", item.shortcut), shortcut_style),
                ])
            })
            .collect();
        frame.render_widget(Paragraph::new(lines), inner);
    }
}
