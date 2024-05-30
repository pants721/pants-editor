use ratatui::style::Style;
use serde::{Deserialize, Serialize};

use super::color::Color;

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
            bg: Color::black().reset(),
            fg: Color::white(),
            statusline_bg: Color::white(),
            statusline_fg: Color::black(),
            color_column: Color::from_rgb(30, 30, 30),
        }
    }
}

impl Theme {
    pub fn primary_style(&self) -> Style {
        Style::default().bg(self.bg.into()).fg(self.fg.into())
    }
}

impl From<Theme> for syntect::highlighting::Theme {
    fn from(value: Theme) -> Self {
        type ThemeSettings = syntect::highlighting::ThemeSettings;
        type SyntectTheme = syntect::highlighting::Theme;
        let ts = ThemeSettings {
            foreground: Some(value.fg.into()),
            background: Some(value.bg.into()),
            ..Default::default()
        };

        SyntectTheme {
            settings: ts,
            ..Default::default()
        }
    }
}
