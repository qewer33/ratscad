use serde::{Deserialize, Serialize};

const APP_NAME: &str = "ratscad";
const CONFIG_NAME: &str = "ratscad";

#[derive(Serialize, Deserialize)]
pub struct Settings {
    pub auto_build: bool,
    pub console_visible: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            auto_build: true,
            console_visible: true,
        }
    }
}

impl Settings {
    pub fn load() -> Self {
        confy::load(APP_NAME, Some(CONFIG_NAME)).unwrap_or_default()
    }

    pub fn save(&self) {
        let _ = confy::store(APP_NAME, Some(CONFIG_NAME), self);
    }
}
