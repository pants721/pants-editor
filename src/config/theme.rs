use ratatui::style::{Color, Style};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Deserialize, Serialize)]
#[serde(default)]
pub struct Theme {
    pub bg: Color,
    pub fg: Color,
    pub statusline_bg: Color,
    pub statusline_fg: Color,
    pub color_column: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            bg: Color::Reset,
            fg: Color::Reset,
            statusline_bg: Color::Red,
            statusline_fg: Color::Black,
            color_column: Color::Rgb(30, 30, 30),
        }
    }
}

impl Theme {
    pub fn primary_style(&self) -> Style {
        Style::default().bg(self.bg).fg(self.fg)
    }
}
