use std::time::Duration;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent};
use ratatui::DefaultTerminal;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use std::path::PathBuf;

use crate::build::BuildCoordinator;
use crate::console::{ConsoleLevel, ConsolePane};
use crate::editor::EditorPane;
use crate::events::{MeshMsg, poll_input};
use crate::menu::{MenuAction, MenuPopup, MenuResult};
use crate::preview::PreviewPane;
use crate::prompt::{Prompt, PromptKind, PromptResult};
use crate::status::GlobalToolbar;

const POLL_TIMEOUT: Duration = Duration::from_millis(50);

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Editor,
    Viewer,
    Menubar,
}

const MENU_ITEMS: [&str; 4] = ["File", "Edit", "View", "Help"];

pub struct App {
    editor: EditorPane,
    preview: PreviewPane,
    build: BuildCoordinator,
    console: ConsolePane,
    focus: Focus,
    menubar_index: usize,
    menu_popup: Option<MenuPopup>,
    prompt: Option<Prompt>,
    fullscreen: bool,
    tab_bar_area: Rect,
    editor_area: Rect,
    preview_area: Rect,
    should_quit: bool,
}

impl App {
    pub fn new() -> anyhow::Result<Self> {
        let editor = EditorPane::new()?;
        let build = BuildCoordinator::spawn();
        build.submit(editor.current_text().to_string());
        Ok(Self {
            editor,
            preview: PreviewPane::new(),
            build,
            console: ConsolePane::new(),
            focus: Focus::Editor,
            menubar_index: 0,
            menu_popup: None,
            prompt: None,
            fullscreen: false,
            tab_bar_area: Rect::default(),
            editor_area: Rect::default(),
            preview_area: Rect::default(),
            should_quit: false,
        })
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> anyhow::Result<()> {
        while !self.should_quit {
            terminal.draw(|frame| self.draw(frame))?;
            self.pump_events()?;
        }
        let _ = self.preview.clear();
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame<'_>) {
        let area = frame.area();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Min(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(area);
        let header_area = chunks[0];
        let tab_bar_area = chunks[1];
        let body_area = chunks[2];
        let pane_toolbar_area = chunks[3];
        let status_area = chunks[4];

        render_header(frame, header_area, self.focus, self.menubar_index);
        self.tab_bar_area = tab_bar_area;
        self.editor.render_tab_bar(frame, tab_bar_area);

        if self.fullscreen {
            self.editor_area = Rect::default();
            self.preview_area = body_area;
            self.preview.render(frame, body_area);
            self.preview
                .render_toolbar(frame, pane_toolbar_area, false, true);
        } else {
            let panes = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
                .split(body_area);
            let pane_toolbars = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
                .split(pane_toolbar_area);
            let editor_split = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(1), Constraint::Length(7)])
                .split(panes[0]);
            self.editor_area = editor_split[0];
            self.preview_area = panes[1];
            self.editor.render(frame, self.editor_area);
            self.console.render(frame, editor_split[1]);
            self.preview.render(frame, self.preview_area);
            if matches!(self.focus, Focus::Editor) {
                self.editor.render_toolbar(frame, pane_toolbars[0]);
            }
            self.preview.render_toolbar(
                frame,
                pane_toolbars[1],
                true,
                matches!(self.focus, Focus::Viewer),
            );
        }

        GlobalToolbar { focus: self.focus }.render(frame, status_area);

        if let Some(menu) = &self.menu_popup {
            menu.render(frame, frame.area());
        }
        if let Some(prompt) = &self.prompt {
            prompt.render(frame, frame.area());
        }
    }

    fn pump_events(&mut self) -> anyhow::Result<()> {
        if let Some(event) = poll_input(POLL_TIMEOUT)? {
            self.handle_input(event)?;
        }
        for msg in self.build.drain() {
            self.handle_mesh(msg)?;
        }
        Ok(())
    }

    fn handle_input(&mut self, event: Event) -> anyhow::Result<()> {
        match event {
            Event::Key(key) => self.handle_key(key)?,
            Event::Mouse(mouse) => self.handle_mouse(mouse)?,
            _ => {}
        }
        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent) -> anyhow::Result<()> {
        if self.prompt.is_some() {
            let result = self.prompt.as_mut().unwrap().on_key(key);
            match result {
                PromptResult::Pending => {}
                PromptResult::Cancelled => self.prompt = None,
                PromptResult::Submitted(text) => {
                    let kind = self.prompt.as_ref().unwrap().kind;
                    self.prompt = None;
                    self.handle_prompt_submit(kind, text)?;
                }
            }
            return Ok(());
        }
        if self.menu_popup.is_some() {
            let result = self.menu_popup.as_mut().unwrap().on_key(key);
            match result {
                MenuResult::Pending => {}
                MenuResult::Cancelled => self.menu_popup = None,
                MenuResult::Activated(action) => {
                    self.menu_popup = None;
                    self.apply_menu_action(action)?;
                }
            }
            return Ok(());
        }

        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('q') => {
                    self.should_quit = true;
                    return Ok(());
                }
                KeyCode::Char('v') => {
                    self.focus = Focus::Viewer;
                    return Ok(());
                }
                KeyCode::Char('e') => {
                    self.focus = Focus::Editor;
                    self.fullscreen = false;
                    return Ok(());
                }
                KeyCode::Char('t') => {
                    self.apply_menu_action(MenuAction::New)?;
                    return Ok(());
                }
                KeyCode::Char('w') => {
                    self.apply_menu_action(MenuAction::Close)?;
                    return Ok(());
                }
                KeyCode::Char('o') => {
                    self.apply_menu_action(MenuAction::Open)?;
                    return Ok(());
                }
                KeyCode::Char('s') => {
                    self.apply_menu_action(MenuAction::Save)?;
                    return Ok(());
                }
                KeyCode::Char('S') => {
                    self.apply_menu_action(MenuAction::SaveAs)?;
                    return Ok(());
                }
                KeyCode::Char(c) => {
                    if let Some(n) = c.to_digit(10) {
                        if (1..=9).contains(&n) {
                            let idx = (n - 1) as usize;
                            if self.editor.switch_to(idx) {
                                self.refresh_preview_for_active()?;
                            }
                            return Ok(());
                        }
                    }
                }
                _ => {}
            }
        }
        if key.modifiers.contains(KeyModifiers::ALT) {
            if key.code == KeyCode::Char('m') {
                self.focus = Focus::Menubar;
                self.menubar_index = 0;
                return Ok(());
            }
            if key.code == KeyCode::Char('h') {
                if self.editor.prev_tab() {
                    self.refresh_preview_for_active()?;
                }
                return Ok(());
            }
            if key.code == KeyCode::Char('l') {
                if self.editor.next_tab() {
                    self.refresh_preview_for_active()?;
                }
                return Ok(());
            }
        }

        match self.focus {
            Focus::Editor => {
                if let Some(source) = self.editor.on_key(key, self.editor_area)? {
                    self.build.submit(source);
                }
            }
            Focus::Viewer => self.handle_viewer_key(key)?,
            Focus::Menubar => self.handle_menubar_key(key),
        }
        Ok(())
    }

    fn handle_viewer_key(&mut self, key: KeyEvent) -> anyhow::Result<()> {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        match key.code {
            KeyCode::Char('f') => self.fullscreen = !self.fullscreen,
            KeyCode::Char('z') => self.preview.zoom(1)?,
            KeyCode::Char('x') => self.preview.zoom(-1)?,
            KeyCode::Up if ctrl => self.preview.pan(0, 1)?,
            KeyCode::Down if ctrl => self.preview.pan(0, -1)?,
            KeyCode::Left if ctrl => self.preview.pan(-1, 0)?,
            KeyCode::Right if ctrl => self.preview.pan(1, 0)?,
            KeyCode::Up => self.preview.rotate(-1, 0)?,
            KeyCode::Down => self.preview.rotate(1, 0)?,
            KeyCode::Left => self.preview.rotate(0, -1)?,
            KeyCode::Right => self.preview.rotate(0, 1)?,
            _ => {}
        }
        Ok(())
    }

    fn handle_menubar_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Left => {
                self.menubar_index = (self.menubar_index + MENU_ITEMS.len() - 1) % MENU_ITEMS.len();
            }
            KeyCode::Right => {
                self.menubar_index = (self.menubar_index + 1) % MENU_ITEMS.len();
            }
            KeyCode::Down | KeyCode::Enter => self.open_active_menu(),
            KeyCode::Esc => self.focus = Focus::Editor,
            _ => {}
        }
    }

    fn open_active_menu(&mut self) {
        if self.menubar_index == 0 {
            let anchor_x = menubar_item_anchor_x(self.menubar_index);
            self.menu_popup = Some(MenuPopup::file_menu(anchor_x, 1));
        }
    }

    fn apply_menu_action(&mut self, action: MenuAction) -> anyhow::Result<()> {
        match action {
            MenuAction::New => {
                self.editor.new_tab()?;
                self.refresh_preview_for_active()?;
                self.focus = Focus::Editor;
            }
            MenuAction::Open => {
                self.prompt = Some(Prompt::open_file());
            }
            MenuAction::Save => {
                if self.editor.active_has_path() {
                    if let Err(e) = self.editor.save_active() {
                        self.set_error(format!("save failed: {e}"));
                    }
                } else {
                    self.prompt = Some(Prompt::save_as(self.editor.active_name()));
                }
            }
            MenuAction::SaveAs => {
                self.prompt = Some(Prompt::save_as(self.editor.active_name()));
            }
            MenuAction::Close => {
                self.editor.close_active_tab();
                self.refresh_preview_for_active()?;
            }
            MenuAction::Quit => {
                self.should_quit = true;
            }
        }
        Ok(())
    }

    fn handle_prompt_submit(&mut self, kind: PromptKind, text: String) -> anyhow::Result<()> {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return Ok(());
        }
        let path = PathBuf::from(trimmed);
        match kind {
            PromptKind::OpenFile => match self.editor.open_file(path) {
                Ok(()) => {
                    self.refresh_preview_for_active()?;
                    self.focus = Focus::Editor;
                }
                Err(e) => {
                    self.set_error(format!("open failed: {e}"));
                }
            },
            PromptKind::SaveAs => {
                if let Err(e) = self.editor.save_active_as(path) {
                    self.set_error(format!("save failed: {e}"));
                }
            }
        }
        Ok(())
    }

    fn handle_mouse(&mut self, mouse: MouseEvent) -> anyhow::Result<()> {
        if hit(self.tab_bar_area, mouse.column, mouse.row) {
            if let crossterm::event::MouseEventKind::Down(_) = mouse.kind {
                if let Some(idx) = self.editor.tab_at_column(mouse.column, self.tab_bar_area) {
                    if self.editor.switch_to(idx) {
                        self.refresh_preview_for_active()?;
                    }
                }
            }
            return Ok(());
        }
        if hit(self.preview_area, mouse.column, mouse.row) {
            self.preview.on_mouse(mouse)?;
        } else if hit(self.editor_area, mouse.column, mouse.row) {
            self.editor.on_mouse(mouse, self.editor_area)?;
        }
        Ok(())
    }

    fn handle_mesh(&mut self, msg: MeshMsg) -> anyhow::Result<()> {
        match msg {
            MeshMsg::Started => {
                self.console.push(ConsoleLevel::Building, "building...");
                self.preview.set_dim(true)?;
            }
            MeshMsg::Ready { source, bytes } => {
                let count = bytes.len();
                self.editor.cache_built_mesh(&source, &bytes);
                if source == self.editor.current_text() {
                    self.preview.register_mesh(&bytes)?;
                    self.preview.set_dim(false)?;
                    self.console
                        .push(ConsoleLevel::Success, format!("ready ({count} bytes)"));
                }
            }
            MeshMsg::Failed(err) => {
                self.preview.set_dim(false)?;
                self.set_error(err);
            }
        }
        Ok(())
    }

    fn set_error(&mut self, message: String) {
        self.console.push(ConsoleLevel::Error, message);
    }

    fn refresh_preview_for_active(&mut self) -> anyhow::Result<()> {
        if let Some(bytes) = self.editor.active_cached_mesh() {
            let bytes = bytes.to_vec();
            self.preview.register_mesh(&bytes)?;
            self.preview.set_dim(false)?;
        } else {
            self.build.submit(self.editor.current_text().to_string());
        }
        Ok(())
    }
}

fn menubar_item_anchor_x(idx: usize) -> u16 {
    // " ratSCAD " (9) + "  │  " (5) = 14, then each " ITEM " (6) + gap "  " (2) = 8
    14 + (idx as u16) * 8
}

fn hit(area: Rect, col: u16, row: u16) -> bool {
    area.width > 0
        && area.height > 0
        && col >= area.x
        && col < area.x.saturating_add(area.width)
        && row >= area.y
        && row < area.y.saturating_add(area.height)
}

fn render_header(frame: &mut Frame<'_>, area: Rect, focus: Focus, menu_index: usize) {
    let menubar_focused = focus == Focus::Menubar;
    let bar_bg = Color::Indexed(236);
    let dim = Style::default().fg(Color::DarkGray).bg(bar_bg);
    let normal = Style::default().fg(Color::Gray).bg(bar_bg);
    let active = Style::default().fg(Color::Yellow).bg(bar_bg);
    let highlight = Style::default()
        .fg(Color::Black)
        .bg(Color::Yellow)
        .add_modifier(Modifier::BOLD);
    let gap = Style::default().bg(bar_bg);

    let mut spans = vec![
        Span::styled(
            " ratSCAD ",
            Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("  │  ", dim),
    ];
    for (i, item) in MENU_ITEMS.iter().enumerate() {
        let style = if menubar_focused && i == menu_index {
            highlight
        } else if menubar_focused {
            active
        } else {
            normal
        };
        if i > 0 {
            spans.push(Span::styled("  ", gap));
        }
        spans.push(Span::styled(format!(" {item} "), style));
    }
    frame.render_widget(
        Paragraph::new(Line::from(spans)).style(Style::default().bg(bar_bg)),
        area,
    );
}

