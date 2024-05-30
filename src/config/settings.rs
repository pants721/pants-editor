use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Deserialize, Serialize)]
pub enum TabType {
    Spaces(usize),
    Tabs(usize),
}

impl Default for TabType {
    fn default() -> Self {
        TabType::Spaces(4)
    }
}

#[derive(Clone, Copy, Deserialize, Serialize)]
#[serde(default)]
pub struct Settings {
    pub line_numbers: bool,
    pub tab_type: TabType,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            line_numbers: true,
            tab_type: TabType::default(),
        }
    }
}
