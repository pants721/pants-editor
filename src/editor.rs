use std::{
    fmt::Display,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Result};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use itertools::Itertools;

use crate::word;

pub enum CursorMove {
    Up,
    Down,
    Left,
    Right,
    LineBegin,
    LineEnd,
    WordStartForward,
    WordStartBackward,
    WordEndForward,
}

#[derive(Default, PartialEq, Eq)]
pub enum EditMode {
    #[default]
    Normal,
    Insert,
}

impl Display for EditMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EditMode::Normal => write!(f, "Normal"),
            EditMode::Insert => write!(f, "Insert"),
        }
    }
}

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

#[derive(Default)]
pub struct Editor {
    pub lines: Vec<String>,
    pub cursor: Cursor,
    pub scroll: (u16, u16),
    pub mode: EditMode,
}

impl Editor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn open_file(&mut self, path: &str) -> Result<()> {
        let path = PathBuf::from(path);
        if !path.is_file() {
            return Err(anyhow!("Path is not file"));
        }
        let file_content = fs::read_to_string(path)?;
        let lines = file_content
            .split('\n')
            .map(|s| s.to_string())
            .collect_vec();
        self.lines = lines;
        Ok(())
    }

    // TODO: I want to make this more of a match mode -> mode.handle_input(input) and mode uses a
    // keymap
    pub fn input(&mut self, key_event: KeyEvent) {
        match self.mode {
            EditMode::Normal => {
                match key_event.code {
                    KeyCode::Char('j') | KeyCode::Down => self.move_cursor(CursorMove::Down),
                    KeyCode::Char('k') | KeyCode::Up => self.move_cursor(CursorMove::Up),
                    KeyCode::Char('h') | KeyCode::Left => self.move_cursor(CursorMove::Left),
                    KeyCode::Char('l') | KeyCode::Right => self.move_cursor(CursorMove::Right),
                    KeyCode::Char('H') | KeyCode::Char('^') => {
                        self.move_cursor(CursorMove::LineBegin)
                    }
                    KeyCode::Char('L') | KeyCode::Char('$') => {
                        self.move_cursor(CursorMove::LineEnd)
                    }
                    KeyCode::Char('w') => self.move_cursor(CursorMove::WordStartForward),
                    KeyCode::Char('b') => self.move_cursor(CursorMove::WordStartBackward),
                    KeyCode::Char('e') => self.move_cursor(CursorMove::WordEndForward),
                    // TODO: This should be dd so find some sort of chord implementation
                    KeyCode::Char('i') => self.mode = EditMode::Insert,
                    KeyCode::Char('I') => {
                        self.move_cursor(CursorMove::LineBegin);
                        // TODO: maybe i should just make a self.insert_mode() function
                        self.mode = EditMode::Insert;
                    }
                    KeyCode::Char('a') => {
                        self.mode = EditMode::Insert;
                        self.move_cursor(CursorMove::Right);
                    }
                    KeyCode::Char('A') => {
                        self.mode = EditMode::Insert;
                        self.move_cursor(CursorMove::LineEnd);
                    }
                    KeyCode::Char('x') => self.delete_char_at_cursor(),
                    KeyCode::Char('X') => self.backspace_at_cursor(),
                    KeyCode::Char('d') => {
                        // TODO: Make this a function. Maybe use a Scroll enum
                        if key_event.modifiers.contains(KeyModifiers::CONTROL) {
                            for _ in 0..19 {
                                self.move_cursor(CursorMove::Down);
                                self.scroll.0 += 1;
                            }
                        } else {
                            self.delete_line_at_cursor();
                        }
                    }
                    KeyCode::Char('u') => {
                        if key_event.modifiers.contains(KeyModifiers::CONTROL) {
                            for _ in 0..19 {
                                self.move_cursor(CursorMove::Up);
                                if self.scroll.0 != 0 {
                                    self.scroll.0 -= 1;
                                }
                            }
                        }
                    }
                    KeyCode::Char('o') => self.newline_under_cursor(),
                    KeyCode::Char('O') => self.newline_above_cursor(),
                    _ => (),
                }
            }
            EditMode::Insert => {
                if key_event.modifiers.contains(KeyModifiers::CONTROL)
                    && key_event.code == KeyCode::Char('c')
                {
                    self.mode = EditMode::Normal;
                    self.move_cursor(CursorMove::Left);
                    return;
                }
                match key_event.code {
                    KeyCode::Down => self.move_cursor(CursorMove::Down),
                    KeyCode::Up => self.move_cursor(CursorMove::Up),
                    KeyCode::Left => self.move_cursor(CursorMove::Left),
                    KeyCode::Right => self.move_cursor(CursorMove::Right),
                    KeyCode::Char(val) => {
                        self.insert_char_at_cursor(val);
                    }
                    KeyCode::Enter => {
                        self.newline_at_cursor();
                    }
                    KeyCode::Backspace => {
                        self.backspace_at_cursor();
                    }
                    KeyCode::Esc => {
                        self.mode = EditMode::Normal;
                        self.move_cursor(CursorMove::Left);
                    }
                    _ => (),
                }
            }
        }
    }

    pub fn insert_char_at_cursor(&mut self, c: char) {
        if let Some(line) = self.lines.get_mut(self.cursor.y) {
            if self.cursor.x == line.len() {
                line.push(c);
            } else {
                line.insert(self.cursor.x, c);
            }

            self.move_cursor(CursorMove::Right);
        }
    }

    pub fn backspace_at_cursor(&mut self) {
        if self.cursor.x == 0 && self.mode == EditMode::Insert {
            if self.cursor.y == 0 {
                return;
            }

            let line = self.lines.remove(self.cursor.y);
            if let Some(prev_line) = self.lines.get_mut(self.cursor.y - 1) {
                let join_idx = prev_line.len();
                prev_line.push_str(&line);
                self.move_cursor(CursorMove::Up);
                self.cursor.x = join_idx;
            }

            return;
        }

        if let Some(line) = self.lines.get_mut(self.cursor.y) {
            if self.cursor.x == line.len() {
                line.pop();
            } else {
                line.remove(self.cursor.x - 1);
            }

            self.move_cursor(CursorMove::Left);
        }
    }

    pub fn delete_char_at_cursor(&mut self) {
        if let Some(line) = self.lines.get_mut(self.cursor.y) {
            if !line.is_empty() {
                line.remove(self.cursor.x);
            }
        }

        if self.char_at(self.cursor.into()).is_none() {
            self.move_cursor(CursorMove::Left);
        }
    }

    pub fn delete_line_at_cursor(&mut self) {
        if self.lines.len() == 1 {
            self.lines[0].clear();
            self.move_cursor(CursorMove::LineBegin);
            return;
        }

        if self.cursor.y == self.lines.len() - 1 {
            self.lines.remove(self.cursor.y);
            self.move_cursor(CursorMove::Up);
            return;
        }

        self.lines.remove(self.cursor.y);
        if self.char_at(self.cursor.into()).is_none() {
            self.move_cursor(CursorMove::LineEnd);
        }
    }

    pub fn newline_above_cursor(&mut self) {
        self.lines.insert(self.cursor.y, "".to_string());
        self.move_cursor(CursorMove::Up);
        self.mode = EditMode::Insert;
    }

    pub fn newline_under_cursor(&mut self) {
        self.lines.insert(self.cursor.y + 1, "".to_string());
        self.move_cursor(CursorMove::Down);
        self.mode = EditMode::Insert;
    }

    pub fn newline_at_cursor(&mut self) {
        if let Some(line) = self.lines.get_mut(self.cursor.y) {
            let line_clone = line.clone();
            let (left, right) = line_clone.split_at(self.cursor.x);
            *line = left.to_string();
            self.lines.insert(self.cursor.y + 1, right.to_string());
            self.move_cursor(CursorMove::Down);
            self.move_cursor(CursorMove::LineBegin);
        }
    }

    pub fn move_cursor(&mut self, cursor_move: CursorMove) {
        match cursor_move {
            // TODO: Implement some sort of column system to mimic vim's vertical movement with
            // lines of different lengths
            CursorMove::Up => {
                let computed = self.cursor.y.saturating_sub(1);
                self.cursor.y = computed;
                if self.char_at((self.cursor.x, computed)).is_none() {
                    self.move_cursor(CursorMove::LineEnd);
                }
            }
            CursorMove::Down => {
                let computed = self.cursor.y.saturating_add(1);
                if self.char_at((self.cursor.x, computed)).is_some() {
                    self.cursor.y = computed;
                } else if computed < self.lines.len() {
                    self.cursor.y = computed;
                    self.move_cursor(CursorMove::LineEnd);
                }
            }
            // XXX: I think this needs to be improved for readability
            CursorMove::Left => {
                let computed = self.cursor.x.saturating_sub(1);
                if self.char_at((computed, self.cursor.y)).is_some() {
                    self.cursor.x = computed;
                } else if let Some(line) = self.lines.get(self.cursor.y) {
                    if computed == line.len() && self.mode == EditMode::Insert {
                        self.cursor.x = computed;
                    }
                }
            }
            // XXX: I think this needs to be improved for readability
            CursorMove::Right => {
                let computed = self.cursor.x.saturating_add(1);
                if self.char_at((computed, self.cursor.y)).is_some() {
                    self.cursor.x = computed;
                } else if let Some(line) = self.lines.get(self.cursor.y) {
                    if computed == line.len() && self.mode == EditMode::Insert {
                        self.cursor.x = computed;
                    }
                }
            }
            CursorMove::LineBegin => {
                self.cursor.x = 0;
            }
            CursorMove::LineEnd => {
                if let Some(line) = self.lines.get(self.cursor.y) {
                    if self.mode == EditMode::Insert {
                        self.cursor.x = line.len();
                    } else {
                        self.cursor.x = line.len().saturating_sub(1);
                    }
                }
            }
            // XXX: At some point this should be replaced by a lexer of some sort
            CursorMove::WordStartForward => {
                if let Some(line) = self.lines.get(self.cursor.y) {
                    self.cursor.x = match word::find_word_start_forward(line, self.cursor.x) {
                        Some(idx) => idx,
                        None => self.cursor.x,
                    }
                }
            }
            CursorMove::WordStartBackward => {
                if let Some(line) = self.lines.get(self.cursor.y) {
                    self.cursor.x = match word::find_word_start_backward(line, self.cursor.x) {
                        Some(idx) => idx,
                        None => self.cursor.x,
                    }
                }
            }
            CursorMove::WordEndForward => {
                if let Some(line) = self.lines.get(self.cursor.y) {
                    self.cursor.x = match word::find_word_end_forward(line, self.cursor.x) {
                        Some(idx) => idx,
                        None => self.cursor.x,
                    }
                }
            }
        }
    }

    pub fn char_at(&self, coords: (usize, usize)) -> Option<char> {
        self.lines.get(coords.1)?.chars().nth(coords.0)
    }
}
