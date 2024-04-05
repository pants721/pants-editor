
use std::process::exit;

use crossterm::event::{KeyCode, KeyEvent};

pub enum CursorMove {
    Up,
    Down,
    Left,
    Right,
    LineBegin, 
    LineEnd, 
}

#[derive(Default, PartialEq, Eq)]
pub enum EditMode {
    #[default]
    Normal,
    Insert,
}

pub struct Editor {
    pub lines: Vec<String>,
    pub cursor: (usize, usize),
    pub mode: EditMode,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            lines: vec![],
            cursor: (0, 0),
            mode: EditMode::default(),
        }
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
                    KeyCode::Char('H') | KeyCode::Char('^') => self.move_cursor(CursorMove::LineBegin),
                    KeyCode::Char('L') | KeyCode::Char('$') => self.move_cursor(CursorMove::LineEnd),
                    KeyCode::Char('i') => self.mode = EditMode::Insert,
                    KeyCode::Char('I') => {
                        self.move_cursor(CursorMove::LineBegin);
                        // TODO: maybe i should just make a self.insert_mode() function
                        self.mode = EditMode::Insert;
                    },
                    KeyCode::Char('a') => {
                        self.mode = EditMode::Insert;
                        self.move_cursor(CursorMove::Right);
                    },
                    KeyCode::Char('A') => {
                        self.mode = EditMode::Insert;
                        self.move_cursor(CursorMove::LineEnd);
                    },
                    KeyCode::Char('x') => self.delete_char_at_cursor(),
                    _ => (),
                }
            },
            EditMode::Insert => {
                match key_event.code {
                    KeyCode::Down => self.move_cursor(CursorMove::Down),
                    KeyCode::Up => self.move_cursor(CursorMove::Up),
                    KeyCode::Left => self.move_cursor(CursorMove::Left),
                    KeyCode::Right => self.move_cursor(CursorMove::Right),
                    KeyCode::Char(val) => {
                        self.insert_char_at_cursor(val);
                    },
                    KeyCode::Backspace => {
                        self.backspace_char_at_cursor();
                    },
                    KeyCode::Esc => {
                        self.mode = EditMode::Normal;
                    },
                    _ => (),
                }
            }
        }
    }

    pub fn insert_char_at_cursor(&mut self, c: char) {
        if let Some(line) = self.lines.get_mut(self.cursor.1) {
            if self.cursor.0 == line.len() {
                line.push(c);
            } else {
                line.insert(self.cursor.0, c);
            }
            
            self.move_cursor(CursorMove::Right);
        }
    }

    pub fn backspace_char_at_cursor(&mut self) {
       if let Some(line)  = self.lines.get_mut(self.cursor.1) {
            if self.cursor.0 == 0 {
                return;
            }

            if self.cursor.0 == line.len() {
                line.pop();
            } else {
                line.remove(self.cursor.0 - 1);
            }
            
            self.move_cursor(CursorMove::Left);
        }
    }

    pub fn delete_char_at_cursor(&mut self) {
       if let Some(line)  = self.lines.get_mut(self.cursor.1) {
            line.remove(self.cursor.0);
        }
    }
    
    pub fn move_cursor(&mut self, cursor_move: CursorMove) {
        match cursor_move {
            CursorMove::Up => {
                let computed = self.cursor.1.saturating_sub(1);
                if self.char_at((self.cursor.0, computed)).is_some() {
                    self.cursor.1 = computed;
                }
            },
            CursorMove::Down => {
                let computed = self.cursor.1.saturating_add(1);
                if self.char_at((self.cursor.0, computed)).is_some() {
                    self.cursor.1 = computed;
                }
            },
            // XXX: I think this needs to be improved for readability
            CursorMove::Left => {
                let computed = self.cursor.0.saturating_sub(1);
                if self.char_at((computed, self.cursor.1)).is_some() {
                    self.cursor.0 = computed;
                } else if let Some(line) = self.lines.get(self.cursor.1) {
                    if computed == line.len() {
                        self.cursor.0 = computed;
                    }
                }
            },
            // XXX: I think this needs to be improved for readability
            CursorMove::Right => {
                let computed = self.cursor.0.saturating_add(1);
                if self.char_at((computed, self.cursor.1)).is_some() {
                    self.cursor.0 = computed;
                } else if let Some(line) = self.lines.get(self.cursor.1) {
                    if computed == line.len() {
                        self.cursor.0 = computed;
                    }
                }
            },
            CursorMove::LineBegin => {
                self.cursor.0 = 0;
            },
            CursorMove::LineEnd => {
                if let Some(line) = self.lines.get(self.cursor.1) {
                    self.cursor.0 = line.len();
                }
            },
        }
    }
    
    pub fn char_at(&self, coords: (usize, usize)) -> Option<char> {
        self.lines.get(coords.1)?.chars().nth(coords.0)
    }
}
