use std::{fs, path::PathBuf};

use anyhow::{anyhow, Result};
use itertools::Itertools;
use ratatui::{prelude::*, widgets::*};

use crate::{
    config::{Settings, TabType},
    cursor::Cursor,
    search::Search,
    util::is_executable,
    word,
};

const MEDIUM_SCROLL: usize = 19;

const COMMAND_HISTORY_MAX: usize = 100;

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
    Start,
    End,
}

#[derive(Default)]
pub enum CurrentScreen {
    #[default]
    Editing,
    Exiting,
}

#[derive(Default)]
pub struct Editor {
    pub lines: Vec<String>,
    pub cursor: Cursor,
    // TODO: Make this absolute path
    pub filename: Option<PathBuf>,
    pub scroll: (u16, u16),
    pub search: Search,
    pub status_message: String,
    pub running: bool,
    pub current_screen: CurrentScreen,
    pub command: String,
    pub command_x: usize,
    pub command_history: Vec<String>,
    pub command_history_idx: usize,
    pub settings: Settings,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            lines: vec!["".to_string()],
            running: true,
            ..Default::default()
        }
    }

    pub fn open(&mut self, path: &str) -> Result<()> {
        let path = PathBuf::from(path);
        if path.exists() && !path.is_file() {
            return Err(anyhow!("Path is not file"));
        }
        if path.exists() && is_executable(&path)? {
            return Err(anyhow!("Cannot open executable"));
        }

        let file_content = fs::read_to_string(&path)?;
        let lines = file_content
            .split('\n')
            .map(|s| s.to_string())
            .collect_vec();
        self.lines = lines;
        self.filename = Some(path);
        Ok(())
    }

    pub fn save(&mut self) -> Result<()> {
        if self.filename.is_none() {
            self.status_message = "Filename not set".to_string();
            return Err(anyhow!("Filename not set"));
        }

        let path = self.filename.as_ref().unwrap();
        let contents = self.lines.join("\n");

        fs::write(path, contents)?;

        self.status_message = format!("\"{}\" written", path.display());
        Ok(())
    }

    pub fn widget(&mut self) -> impl Widget + '_ {
        Renderer::new(self)
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
        if self.cursor.x == 0 {
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
        self.move_cursor(CursorMove::LineBegin);
    }

    pub fn newline_under_cursor(&mut self) {
        self.lines.insert(self.cursor.y + 1, "".to_string());
        self.move_cursor(CursorMove::Down);
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

    pub fn insert_tab(&mut self) {
        if let Some(line) = self.lines.get_mut(self.cursor.y) {
            match self.settings.tab_type {
                TabType::Spaces(n) => {
                    line.insert_str(self.cursor.x, &" ".repeat(n));
                    self.cursor.x += n;
                }
                TabType::Tabs(n) => {
                    line.insert_str(self.cursor.x, &"\t".repeat(n));
                    self.cursor.x += n;
                }
            }
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
                    if computed == line.len() {
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
                    if computed == line.len() {
                        self.cursor.x = computed;
                    }
                }
            }
            CursorMove::LineBegin => {
                self.cursor.x = 0;
            }
            CursorMove::LineEnd => {
                if let Some(line) = self.lines.get(self.cursor.y) {
                    self.cursor.x = line.len();
                }
            }
            // XXX: At some point this should be replaced by a lexer of some sort
            // TODO: Make this go next line
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
            CursorMove::Start => {
                self.cursor = (0, 0).into();
            }
            CursorMove::End => {
                let last_line = self.lines.len().saturating_sub(1);
                let last_line_len = self.lines.get(last_line).map(|l| l.len()).unwrap_or(0);
                self.cursor = (last_line_len, last_line).into();
            }
        }
    }

    pub fn move_command_cursor(&mut self, cursor_move: CursorMove) {
        match cursor_move {
            CursorMove::Left => {
                self.command_x = self.command_x.saturating_sub(1);
            }
            CursorMove::Right => {
                self.command_x = (self.command_x + 1).clamp(0, self.command.len());
            }
            CursorMove::LineBegin => {
                self.command_x = 0;
            }
            CursorMove::LineEnd => {
                self.command_x = self.command.len();
            }
            _ => {}
        }
    }

    pub fn scroll_up(&mut self, amount: usize) {
        self.scroll.0 = self.scroll.0.saturating_sub(amount as u16);
        self.cursor.y = self.cursor.y.saturating_sub(amount);
    }

    pub fn scroll_down(&mut self, amount: usize) {
        self.scroll.0 = (self.scroll.0 + amount as u16).clamp(0, self.lines.len() as u16 - 1);
        self.cursor.y = (self.cursor.y + amount).clamp(0, self.lines.len() - 1);
    }

    pub fn med_scroll_up(&mut self) {
        self.scroll_up(MEDIUM_SCROLL);
    }

    pub fn med_scroll_down(&mut self) {
        self.scroll_down(MEDIUM_SCROLL);
    }

    pub fn clear_search(&mut self) {
        self.search.query.clear();
        self.command_x = 0;
    }

    pub fn execute_current_search(&mut self) {
        self.search.search(&self.lines);
        if self.search.results.is_empty() {
            self.status_message = "Pattern not found".to_string();
            return;
        }
        if let Some(first_result) = self.search.results.first() {
            self.cursor = (first_result.start, first_result.row).into();
            if let Some((idx, _)) = self
                .search
                .results
                .iter()
                .enumerate()
                .find(|(_, r)| r.row == self.cursor.y)
            {
                self.status_message = format!(
                    "/{} - [{}/{}]",
                    self.search.query,
                    idx + 1,
                    self.search.results.len()
                );
            }
        }
    }

    pub fn search_next(&mut self) {
        if let Some((idx, _)) = self
            .search
            .results
            .iter()
            .enumerate()
            .find(|(_, r)| r.row == self.cursor.y)
        {
            let (idx, next) = if idx == self.search.results.len() - 1 {
                let idx = 0;
                (idx, &self.search.results[idx])
            } else {
                let idx = idx + 1;
                (idx, &self.search.results[idx])
            };
            self.cursor = (next.start, next.row).into();
            self.status_message = format!(
                "/{} - [{}/{}]",
                self.search.query,
                idx + 1,
                self.search.results.len()
            );
        }
    }

    pub fn search_prev(&mut self) {
        if let Some((idx, _)) = self
            .search
            .results
            .iter()
            .enumerate()
            .find(|(_, r)| r.row == self.cursor.y)
        {
            let (idx, prev) = if idx == 0 {
                let idx = self.search.results.len() - 1;
                (idx, &self.search.results[idx])
            } else {
                let idx = idx - 1;
                (idx, &self.search.results[idx])
            };
            self.cursor = (prev.start, prev.row).into();
            self.status_message = format!(
                "/{} - [{}/{}]",
                self.search.query,
                idx + 1,
                self.search.results.len()
            );
        }
    }

    pub fn char_at(&self, coords: (usize, usize)) -> Option<char> {
        self.lines.get(coords.1)?.chars().nth(coords.0)
    }

    pub fn is_dirty(&self) -> Result<bool> {
        if let Some(filename) = &self.filename {
            let disk_file_content = fs::read_to_string(filename)?;

            return Ok(self.lines.join("\n") != disk_file_content);
        }

        Ok(false)
    }
}

struct Renderer<'a> {
    editor: &'a mut Editor,
}

impl<'a> Renderer<'a> {
    pub fn new(editor: &'a mut Editor) -> Self {
        Self { editor }
    }
}

impl<'a> Widget for Renderer<'a> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        if (self.editor.cursor.y as u16) < self.editor.scroll.0 {
            self.editor.scroll.0 = self.editor.cursor.y as u16;
        } else if (self.editor.cursor.y as u16) >= self.editor.scroll.0 + area.height {
            self.editor.scroll.0 = (self.editor.cursor.y as u16) - area.height.saturating_sub(1);
        }

        // XXX: perf
        let text_block = Paragraph::new(self.editor.lines.join("\n"))
            .scroll(self.editor.scroll);

        text_block.render(area, buf);
    }
}
