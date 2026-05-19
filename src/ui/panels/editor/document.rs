use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use ratatui_code_editor::editor::Editor;

const OPENSCAD_HIGHLIGHTS: &str = include_str!("openscad_highlights.scm");

fn highlights_for_c() -> Option<HashMap<String, String>> {
    let mut map = HashMap::new();
    map.insert("c".to_string(), OPENSCAD_HIGHLIGHTS.to_string());
    Some(map)
}

// Mostly vesper(), with function captures repainted purple so they don't
// share vesper's orange with numbers, constants and types.
fn ratscad_theme() -> Vec<(&'static str, &'static str)> {
    vec![
        ("identifier", "#A5FCB6"),
        ("field_identifier", "#A5FCB6"),
        ("property_identifier", "#A5FCB6"),
        ("property", "#A5FCB6"),
        ("string", "#b1fce5"),
        ("keyword", "#a0a0a0"),
        ("constant", "#f6c99f"),
        ("number", "#f6c99f"),
        ("integer", "#f6c99f"),
        ("float", "#f6c99f"),
        ("variable", "#ffffff"),
        ("variable.builtin", "#ffffff"),
        ("function", "#c6a5fc"),
        ("function.call", "#c6a5fc"),
        ("method", "#c6a5fc"),
        ("function.macro", "#c6a5fc"),
        ("comment", "#585858"),
        ("namespace", "#f6c99f"),
        ("type", "#f6c99f"),
        ("type.builtin", "#f6c99f"),
        ("tag.attribute", "#c6a5fc"),
        ("tag", "#c6a5fc"),
        ("error", "#A5FCB6"),
        ("diff_added", "#017d4e"),
        ("diff_deleted", "#d94b4b"),
    ]
}

pub struct Document {
    pub name: String,
    pub path: Option<PathBuf>,
    pub editor: Editor,
    pub last_text: String,
    pub dirty: bool,
    pub cached: Option<(String, Vec<u8>)>,
}

impl Document {
    pub fn new_untitled(name: impl Into<String>, content: &str) -> anyhow::Result<Self> {
        let editor = Editor::new_with_highlights("c", content, ratscad_theme(), highlights_for_c())?;
        Ok(Self {
            name: name.into(),
            path: None,
            editor,
            last_text: content.to_string(),
            dirty: true,
            cached: None,
        })
    }

    pub fn open(path: PathBuf) -> anyhow::Result<Self> {
        let content = fs::read_to_string(&path)?;
        let name = file_name(&path);
        let editor = Editor::new_with_highlights("c", &content, ratscad_theme(), highlights_for_c())?;
        Ok(Self {
            name,
            path: Some(path),
            editor,
            last_text: content,
            dirty: false,
            cached: None,
        })
    }

    pub fn save(&mut self) -> anyhow::Result<bool> {
        let Some(path) = self.path.clone() else {
            return Ok(false);
        };
        fs::write(&path, &self.last_text)?;
        self.dirty = false;
        Ok(true)
    }

    pub fn save_as(&mut self, path: PathBuf) -> anyhow::Result<()> {
        fs::write(&path, &self.last_text)?;
        self.name = file_name(&path);
        self.path = Some(path);
        self.dirty = false;
        Ok(())
    }
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .and_then(|n| n.to_str())
        .map(String::from)
        .unwrap_or_else(|| "untitled".to_string())
}
