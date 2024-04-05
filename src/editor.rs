use std::{fs, path::Path};

use anyhow::{anyhow, Result};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use itertools::Itertools;

pub enum CursorMove {
    Up,
    Down,
    Left,
    Right,
    LineBegin, 
    LineEnd, 
    WordForward,
    WordBackward,
}

#[derive(Default, PartialEq, Eq)]
pub enum EditMode {
    #[default]
    Normal,
    Insert,
}

#[derive(Default)]
pub struct Editor {
    pub lines: Vec<String>,
    pub cursor: (usize, usize),
    pub mode: EditMode,
}

impl Editor {
    pub fn new() -> Self {
        Self::default()
    }

    #[allow(dead_code)]
    pub fn open_file(path: &Path) -> Result<Self> {
        if !path.is_file() {
            return Err(anyhow!("Path is not file"));
        }
        let file_content = fs::read_to_string(path)?;
        let lines = file_content.split('\n').map(|s| s.to_string()).collect_vec();

        Ok(Self {
            lines,
            ..Default::default()
        })
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
                    KeyCode::Char('w') => self.move_cursor(CursorMove::WordForward),
                    KeyCode::Char('b') => self.move_cursor(CursorMove::WordBackward),
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
                    KeyCode::Char('o') => self.newline_under_cursor(),
                    KeyCode::Char('O') => self.newline_above_cursor(),
                    _ => (),
                }
            },
            EditMode::Insert => {
                if key_event.modifiers.contains(KeyModifiers::CONTROL) && key_event.code == KeyCode::Char('c') {
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
                    },
                    KeyCode::Enter => {
                        self.newline_at_cursor();
                    },
                    KeyCode::Backspace => {
                        self.backspace_at_cursor();
                    },
                    KeyCode::Esc => {
                        self.mode = EditMode::Normal;
                        self.move_cursor(CursorMove::Left);
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

    pub fn backspace_at_cursor(&mut self) {
        if self.cursor.0 == 0 {
            if self.cursor.1 == 0 {
                return;
            }
            
            let line = self.lines.remove(self.cursor.1);
            if let Some(prev_line) = self.lines.get_mut(self.cursor.1 - 1) {
                let join_idx = prev_line.len();
                prev_line.push_str(&line);
                self.move_cursor(CursorMove::Up);
                self.cursor.0 = join_idx;
            }
            
            return;
        }
        
        if let Some(line)  = self.lines.get_mut(self.cursor.1) {

            if self.cursor.0 == line.len() {
                line.pop();
            } else {
                line.remove(self.cursor.0 - 1);
            }
            
            self.move_cursor(CursorMove::Left);
        }
    }

    pub fn delete_char_at_cursor(&mut self) {
       if let Some(line) = self.lines.get_mut(self.cursor.1) {
            if !line.is_empty() {
                line.remove(self.cursor.0);
            }
        }
    }

    pub fn newline_above_cursor(&mut self) {
        self.lines.insert(self.cursor.1, "".to_string());
        self.move_cursor(CursorMove::Up);
        self.mode = EditMode::Insert;
    }  
    
    pub fn newline_under_cursor(&mut self) {
        self.lines.insert(self.cursor.1 + 1, "".to_string());
        self.move_cursor(CursorMove::Down);
        self.mode = EditMode::Insert;
    }

    pub fn newline_at_cursor(&mut self) {
        if let Some(line) = self.lines.get_mut(self.cursor.1) {
            let line_clone = line.clone();
            let (left, right) = line_clone.split_at(self.cursor.0);
            *line = left.to_string();
            self.lines.insert(self.cursor.1 + 1, right.to_string());
            self.move_cursor(CursorMove::Down);
            self.move_cursor(CursorMove::LineBegin);
        }
    }
    
    pub fn move_cursor(&mut self, cursor_move: CursorMove) {
        match cursor_move {
            // TODO: Implement some sort of column system to mimic vim's vertical movement with
            // lines of different lengths
            CursorMove::Up => {
                let computed = self.cursor.1.saturating_sub(1);
                if self.char_at((self.cursor.0, computed)).is_some() {
                    self.cursor.1 = computed;
                } else {
                    self.cursor.1 = computed;
                    self.move_cursor(CursorMove::LineEnd);
                }
            },
            CursorMove::Down => {
                let computed = self.cursor.1.saturating_add(1);
                if self.char_at((self.cursor.0, computed)).is_some() {
                    self.cursor.1 = computed;
                } else if computed < self.lines.len() {
                    self.cursor.1 = computed;
                    self.move_cursor(CursorMove::LineEnd);
                }
            },
            // XXX: I think this needs to be improved for readability
            CursorMove::Left => {
                let computed = self.cursor.0.saturating_sub(1);
                if self.char_at((computed, self.cursor.1)).is_some() {
                    self.cursor.0 = computed;
                } else if let Some(line) = self.lines.get(self.cursor.1) {
                    if computed == line.len() && self.mode == EditMode::Insert {
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
                    if computed == line.len() && self.mode == EditMode::Insert {
                        self.cursor.0 = computed;
                    }
                }
            },
            CursorMove::LineBegin => {
                self.cursor.0 = 0;
            },
            CursorMove::LineEnd => {
                if let Some(line) = self.lines.get(self.cursor.1) {
                    if self.mode == EditMode::Insert {
                        self.cursor.0 = line.len();
                    } else {
                        self.cursor.0 = line.len().saturating_sub(1);
                    }
                }
            },
            // XXX: At some point this   should be replaced by a lexer of some sort
            CursorMove::WordForward => {
                if let Some(line) = self.lines.get(self.cursor.1) {
                    let ws_dist = line[self.cursor.0..].find(|c: char| c.is_whitespace()).unwrap_or(0);
                    let alpha_dist = line[self.cursor.0 + ws_dist..].find(|c: char| !c.is_whitespace()).unwrap_or(0);
                    self.cursor.0 += ws_dist + alpha_dist;
                }
            },
            CursorMove::WordBackward => {
                if let Some(line) = self.lines.get(self.cursor.1) {
                    let alpha_dist = line[..self.cursor.0].rfind(|c: char| !c.is_whitespace()).unwrap_or(0);
                    let start_dist = line[..alpha_dist].rfind(|c: char| c.is_whitespace()).map(|i| i + 1).unwrap_or(0);
                    self.cursor.0 = start_dist;
                }
            },
        }
    }

    pub fn char_at(&self, coords: (usize, usize)) -> Option<char> {
        self.lines.get(coords.1)?.chars().nth(coords.0)
    }
}

