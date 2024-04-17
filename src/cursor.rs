#[derive(Clone, Copy, Debug, Default)]
pub struct Cursor {
    pub x: usize,
    pub y: usize,
}

impl From<Cursor> for (usize, usize) {
    fn from(value: Cursor) -> Self {
        (value.x, value.y)
    }
}

impl From<(usize, usize)> for Cursor {
    fn from(value: (usize, usize)) -> Self {
        Self {
            x: value.0,
            y: value.1,
        }
    }
}
