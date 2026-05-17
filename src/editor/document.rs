use std::fs;
use std::path::{Path, PathBuf};

use ratatui_code_editor::editor::Editor;
use ratatui_code_editor::theme::vesper;

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
        let editor = Editor::new("c", content, vesper())?;
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
        let editor = Editor::new("c", &content, vesper())?;
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
