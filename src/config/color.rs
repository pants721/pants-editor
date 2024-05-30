use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Default, Deserialize, Serialize)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
    reset: bool,
}

impl Color {
    pub fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            r,
            g,
            b,
            a,
            ..Default::default()
        }
    }
    
    /// a is 255
    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self {
            r,
            g,
            b,
            a: 255,
            ..Default::default()
        }
    }

    pub fn black() -> Self {
        Self::from_rgb(0, 0, 0)
    }
    
    pub fn white() -> Self {
        Self::from_rgb(255, 255, 255)
    }

    /// sets as a terminal reset color
    pub fn reset(mut self) -> Color {
        self.reset = true;
        self
    }
}

impl From<Color> for ratatui::prelude::Color {
    fn from(value: Color) -> Self {
        if value.reset {
            Self::Reset
        } else {
            Self::Rgb(value.r, value.g, value.b)
        }
    }
}

impl From<Color> for syntect::highlighting::Color {
    fn from(value: Color) -> Self {
        Self {
            r: value.r,
            g: value.g,
            b: value.b,
            a: value.a,
        }
    }
}
