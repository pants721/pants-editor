use serde::{Deserialize, Serialize};

use super::theme::Theme;

#[derive(Clone, Copy, Deserialize, Serialize)]
pub struct Settings {
    pub line_numbers: bool, 
    #[serde(default)]
    pub theme: Theme,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            line_numbers: true,
            theme: Theme::default(),
        }
    }
}
