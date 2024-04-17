use std::{
    io::{self, stdout},
    panic::{set_hook, take_hook},
};

use anyhow::{Context, Result};
use config::Settings;
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use editor::{CurrentScreen, CursorMove, EditMode, Editor};
use figment::{
    providers::{Format, Toml},
    Figment,
};
use itertools::Itertools;
use ratatui::prelude::*;
use ui::ui;
use util::pe_config_file_path;

mod command;
mod config;
mod cursor;
mod editor;
mod renderer;
mod search;
mod ui;
mod util;
mod word;

fn main() -> Result<()> {
    let args = std::env::args();

    let mut editor = Editor::new();
    let config: Settings = Figment::new()
        .merge(Toml::file(pe_config_file_path()?))
        .extract()
        .context("Failed to load config")?;
    editor.settings = config;

    if args.len() > 1 {
        editor.open(&args.collect_vec()[1])?;
    }

    install_panic_hook();
    let mut terminal = init_terminal()?;

    while editor.running {
        terminal.draw(|f| {
            ui(f, &mut editor);
        })?;
        handle_event(&mut editor)?;
    }

    terminal.show_cursor()?;
    restore_terminal()?;

    Ok(())
}

fn handle_event(editor: &mut Editor) -> Result<()> {
    if let Event::Key(key) = event::read()? {
        if key.kind == event::KeyEventKind::Release {
            return Ok(());
        }

        handle_key(key, editor)?;
    }
    Ok(())
}

fn handle_key(key: KeyEvent, editor: &mut Editor) -> Result<()> {
    match editor.current_screen {
        CurrentScreen::Editing => {
            match editor.mode {
                EditMode::Normal => {
                    match key.code {
                        KeyCode::Char('j') | KeyCode::Down => editor.move_cursor(CursorMove::Down),
                        KeyCode::Char('k') | KeyCode::Up => editor.move_cursor(CursorMove::Up),
                        KeyCode::Char('h') | KeyCode::Left => editor.move_cursor(CursorMove::Left),
                        KeyCode::Char('l') | KeyCode::Right => {
                            editor.move_cursor(CursorMove::Right)
                        }
                        KeyCode::Char('H') | KeyCode::Char('^') => {
                            editor.move_cursor(CursorMove::LineBegin)
                        }
                        KeyCode::Char('L') | KeyCode::Char('$') => {
                            editor.move_cursor(CursorMove::LineEnd)
                        }
                        KeyCode::Char('g') => editor.move_cursor(CursorMove::Start),
                        KeyCode::Char('G') => editor.move_cursor(CursorMove::End),
                        KeyCode::Char('w') => editor.move_cursor(CursorMove::WordStartForward),
                        KeyCode::Char('b') => editor.move_cursor(CursorMove::WordStartBackward),
                        KeyCode::Char('e') => editor.move_cursor(CursorMove::WordEndForward),
                        // TODO: This should be dd so find some sort of chord implementation
                        KeyCode::Char('i') => editor.mode = EditMode::Insert,
                        KeyCode::Char('I') => {
                            editor.move_cursor(CursorMove::LineBegin);
                            editor.mode = EditMode::Insert;
                        }
                        KeyCode::Char('a') => {
                            editor.mode = EditMode::Insert;
                            editor.move_cursor(CursorMove::Right);
                        }
                        KeyCode::Char('A') => {
                            editor.mode = EditMode::Insert;
                            editor.move_cursor(CursorMove::LineEnd);
                        }
                        KeyCode::Char('x') => editor.delete_char_at_cursor(),
                        KeyCode::Char('X') => editor.backspace_at_cursor(),
                        KeyCode::Char('u') => {
                            if key.modifiers.contains(KeyModifiers::CONTROL) {
                                editor.med_scroll_up();
                            }
                        }
                        KeyCode::Char('d') => {
                            if key.modifiers.contains(KeyModifiers::CONTROL) {
                                editor.med_scroll_down();
                            } else {
                                editor.delete_line_at_cursor();
                            }
                        }
                        KeyCode::Char('s') => {
                            if key.modifiers.contains(KeyModifiers::CONTROL) {
                                editor.save()?;
                            }
                        }
                        KeyCode::Char('o') => editor.newline_under_cursor(),
                        KeyCode::Char('O') => editor.newline_above_cursor(),
                        KeyCode::Char(':') => editor.mode = EditMode::Command,
                        KeyCode::Char('/') => {
                            editor.search.query.clear();
                            editor.mode = EditMode::Search;
                        }
                        KeyCode::Char('n') => editor.search_next(),
                        KeyCode::Char('N') => editor.search_prev(),
                        _ => (),
                    }
                }
                EditMode::Insert => {
                    if key.modifiers.contains(KeyModifiers::CONTROL)
                        && key.code == KeyCode::Char('c')
                    {
                        editor.mode = EditMode::Normal;
                        editor.move_cursor(CursorMove::Left);
                        return Ok(());
                    }
                    match key.code {
                        KeyCode::Down => editor.move_cursor(CursorMove::Down),
                        KeyCode::Up => editor.move_cursor(CursorMove::Up),
                        KeyCode::Left => editor.move_cursor(CursorMove::Left),
                        KeyCode::Right => editor.move_cursor(CursorMove::Right),
                        KeyCode::Char(val) => {
                            editor.insert_char_at_cursor(val);
                        }
                        KeyCode::Enter => {
                            editor.newline_at_cursor();
                        }
                        KeyCode::Backspace => {
                            editor.backspace_at_cursor();
                        }
                        KeyCode::Esc => {
                            editor.mode = EditMode::Normal;
                            editor.move_cursor(CursorMove::Left);
                        }
                        KeyCode::Tab => {
                            editor.insert_tab();
                        }
                        _ => (),
                    }
                }
                EditMode::Command => {
                    if key.modifiers.contains(KeyModifiers::CONTROL)
                        && key.code == KeyCode::Char('c')
                    {
                        editor.mode = EditMode::Normal;
                        return Ok(());
                    }

                    match key.code {
                        KeyCode::Esc => editor.mode = EditMode::Normal,
                        KeyCode::Char(c) => {
                            editor.insert_char_in_command(c);
                        }
                        KeyCode::Backspace => editor.backspace_char_in_command(),
                        KeyCode::Enter => editor.execute_current_command()?,
                        _ => (),
                    }
                }
                EditMode::Search => {
                    if key.modifiers.contains(KeyModifiers::CONTROL)
                        && key.code == KeyCode::Char('c')
                    {
                        editor.mode = EditMode::Normal;
                        return Ok(());
                    }
                    match key.code {
                        KeyCode::Char(c) => {
                            editor.insert_char_in_search(c);
                        }
                        KeyCode::Backspace => editor.backspace_char_in_search(),
                        KeyCode::Enter => {
                            editor.execute_current_search();
                        }
                        _ => (),
                    }
                }
            }
        }
        CurrentScreen::Exiting => match key.code {
            KeyCode::Char('y') => {
                editor.running = false;
            }
            KeyCode::Char('n') | KeyCode::Char('q') => {
                editor.current_screen = CurrentScreen::Editing;
            }
            _ => {}
        },
    }
    Ok(())
}

pub fn init_terminal() -> Result<Terminal<impl Backend>> {
    enable_raw_mode()?;
    let mut stderr = io::stderr();
    execute!(stderr, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout());
    let terminal = Terminal::new(backend)?;

    Ok(terminal)
}

pub fn restore_terminal() -> Result<()> {
    io::stdout().execute(LeaveAlternateScreen)?;
    io::stdout().execute(DisableMouseCapture)?;
    disable_raw_mode()?;
    Ok(())
}

pub fn install_panic_hook() {
    let original_hook = take_hook();
    set_hook(Box::new(move |panic_info| {
        io::stdout().execute(LeaveAlternateScreen).unwrap();
        io::stdout().execute(DisableMouseCapture).unwrap();
        disable_raw_mode().unwrap();
        original_hook(panic_info);
    }));
}
