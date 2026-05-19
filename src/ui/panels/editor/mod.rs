mod document;

use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::ui::theme::{desc_style, key_style, sep_style};

use self::document::Document;

const CHEESE_SCAD: &str = "// ratSCAD cheese demo
$fn = 50;

// Cheese!
color(\"yellow\") mirror([1, 0, 0]) difference() {
    // Base triangle
    linear_extrude(13) polygon([[0, 0], [10, 25], [20, 0]]);

    // Cut spheres
    union() {
        translate([14,3, 15]) sphere(3);
        translate([10, 6, 14]) sphere(3);
        translate([5, 16, 14]) sphere(5);
        translate([-1, 6, 0]) sphere(5);
        translate([3, 17, 3]) sphere(4);
    }
}
";

pub struct EditorPane {
    documents: Vec<Document>,
    active: usize,
}

impl EditorPane {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            documents: vec![Document::new_untitled("cheese.scad", CHEESE_SCAD)?],
            active: 0,
        })
    }

    pub fn current_text(&self) -> &str {
        &self.documents[self.active].last_text
    }

    pub fn on_key(&mut self, key: KeyEvent, area: Rect) -> anyhow::Result<Option<String>> {
        let doc = &mut self.documents[self.active];
        doc.editor.input(key, &area)?;
        let text = doc.editor.get_content();
        if text != doc.last_text {
            doc.last_text = text.clone();
            doc.dirty = true;
            Ok(Some(text))
        } else {
            Ok(None)
        }
    }

    pub fn on_mouse(&mut self, mouse: MouseEvent, area: Rect) -> anyhow::Result<()> {
        self.documents[self.active].editor.mouse(mouse, &area)?;
        Ok(())
    }

    pub fn render(&self, frame: &mut Frame<'_>, area: Rect) {
        let doc = &self.documents[self.active];
        frame.render_widget(&doc.editor, area);
        if let Some((x, y)) = doc.editor.get_visible_cursor(&area) {
            frame.set_cursor_position(ratatui::layout::Position::new(x, y));
        }
    }

    pub fn new_tab(&mut self) -> anyhow::Result<()> {
        let n = self.next_untitled_number();
        let name = format!("untitled-{n}.scad");
        self.documents.push(Document::new_untitled(name, "")?);
        self.active = self.documents.len() - 1;
        Ok(())
    }

    pub fn close_active_tab(&mut self) {
        if self.documents.len() <= 1 {
            return;
        }
        self.documents.remove(self.active);
        if self.active >= self.documents.len() {
            self.active = self.documents.len() - 1;
        }
    }

    pub fn open_file(&mut self, path: std::path::PathBuf) -> anyhow::Result<()> {
        let doc = Document::open(path)?;
        self.documents.push(doc);
        self.active = self.documents.len() - 1;
        Ok(())
    }

    pub fn save_active(&mut self) -> anyhow::Result<bool> {
        self.documents[self.active].save()
    }

    pub fn save_active_as(&mut self, path: std::path::PathBuf) -> anyhow::Result<()> {
        self.documents[self.active].save_as(path)
    }

    pub fn active_has_path(&self) -> bool {
        self.documents[self.active].path.is_some()
    }

    pub fn active_name(&self) -> &str {
        &self.documents[self.active].name
    }

    pub fn active_cached_mesh(&self) -> Option<&[u8]> {
        let doc = &self.documents[self.active];
        doc.cached.as_ref().and_then(|(src, bytes)| {
            (src == &doc.last_text).then_some(bytes.as_slice())
        })
    }

    pub fn cache_built_mesh(&mut self, source: &str, bytes: &[u8]) {
        for doc in &mut self.documents {
            if doc.last_text == source {
                doc.cached = Some((source.to_string(), bytes.to_vec()));
            }
        }
    }

    pub fn switch_to(&mut self, idx: usize) -> bool {
        if idx < self.documents.len() && idx != self.active {
            self.active = idx;
            true
        } else {
            false
        }
    }

    pub fn next_tab(&mut self) -> bool {
        if self.documents.len() <= 1 {
            return false;
        }
        self.active = (self.active + 1) % self.documents.len();
        true
    }

    pub fn prev_tab(&mut self) -> bool {
        if self.documents.len() <= 1 {
            return false;
        }
        self.active = (self.active + self.documents.len() - 1) % self.documents.len();
        true
    }

    pub fn render_tab_bar(&self, frame: &mut Frame<'_>, area: Rect) {
        let active_style = Style::default()
            .fg(Color::Black)
            .bg(Color::Yellow)
            .add_modifier(Modifier::BOLD);
        let inactive_style = Style::default().fg(Color::Gray);
        let dirty_color_active = Color::Black;
        let dirty_color_inactive = Color::LightRed;

        let mut spans = Vec::new();
        for (i, doc) in self.documents.iter().enumerate() {
            let active = i == self.active;
            let style = if active { active_style } else { inactive_style };
            let dirty = doc.dirty;
            let dirty_color = if active {
                dirty_color_active
            } else {
                dirty_color_inactive
            };
            spans.push(Span::styled(" ", style));
            if dirty {
                spans.push(Span::styled("●", style.fg(dirty_color)));
            } else {
                spans.push(Span::styled(" ", style));
            }
            spans.push(Span::styled(format!(" {} ", doc.name), style));
            spans.push(Span::raw(" "));
        }
        frame.render_widget(Paragraph::new(Line::from(spans)), area);
    }

    pub fn render_toolbar(&self, frame: &mut Frame<'_>, area: Rect) {
        let key = key_style();
        let desc = desc_style();
        let sep = sep_style();
        let spans = vec![
            Span::raw(" "),
            Span::styled("Ctrl", sep),
            Span::styled(" + ", sep),
            Span::styled(" t ", key),
            Span::styled(" new tab ", desc),
            Span::styled(" w ", key),
            Span::styled(" close ", desc),
            Span::styled("   ", desc),
            Span::styled("Alt", sep),
            Span::styled(" + ", sep),
            Span::styled(" h ", key),
            Span::styled(" prev ", desc),
            Span::styled(" l ", key),
            Span::styled(" next", desc),
        ];
        frame.render_widget(Paragraph::new(Line::from(spans)), area);
    }

    pub fn tab_at_column(&self, col: u16, area: Rect) -> Option<usize> {
        if col < area.x {
            return None;
        }
        let local = col - area.x;
        let mut cursor: u16 = 0;
        for (i, doc) in self.documents.iter().enumerate() {
            let width = tab_width(&doc.name);
            if local >= cursor && local < cursor + width {
                return Some(i);
            }
            cursor = cursor.saturating_add(width).saturating_add(1);
        }
        None
    }
}

fn tab_width(name: &str) -> u16 {
    // Each tab renders as: leading space, dirty mark, space, name, trailing
    // space. Four constant cells around the name.
    (name.chars().count() as u16).saturating_add(4)
}

impl EditorPane {
    fn next_untitled_number(&self) -> usize {
        self.documents
            .iter()
            .filter_map(|d| {
                d.name
                    .strip_prefix("untitled-")
                    .and_then(|s| s.strip_suffix(".scad"))
                    .and_then(|s| s.parse::<usize>().ok())
            })
            .max()
            .map_or(1, |n| n + 1)
    }
}
