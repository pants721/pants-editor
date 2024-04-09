#[derive(Clone, Copy, Default)]
pub struct Cursor {
    pub x: usize,
    pub y: usize,
}

impl From<Cursor> for (usize, usize) {
    fn from(value: Cursor) -> Self {
        (value.x, value.y)
    }
}
