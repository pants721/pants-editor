use std::{
    io::{self, stdout, Stderr, Stdout}, panic::{set_hook, take_hook}, path::PathBuf
};

use anyhow::{anyhow, Result};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}, ExecutableCommand,
};
use editor::{CurrentScreen, CursorMove, EditMode, Editor};
use itertools::Itertools;
use ratatui::prelude::*;
use ui::ui;

mod cursor;
mod editor;
mod ui;
mod word;
mod renderer;

fn main() -> Result<()> {
    install_panic_hook();
    let mut terminal = init_terminal()?;

    let args = std::env::args();

    let mut editor = Editor::new();
    if args.len() > 1 {
        editor.open(&args.collect_vec()[1])?;
    }

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

            if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('q') {
                if editor.is_dirty()? {
                    editor.current_screen = CurrentScreen::Exiting;
                    return Ok(());
                }

                editor.running = false;
                return Ok(());

            }

            match editor.mode {
                EditMode::Normal => {
                    match key.code {
                        KeyCode::Char('j') | KeyCode::Down => editor.move_cursor(CursorMove::Down),
                        KeyCode::Char('k') | KeyCode::Up => editor.move_cursor(CursorMove::Up),
                        KeyCode::Char('h') | KeyCode::Left => editor.move_cursor(CursorMove::Left),
                        KeyCode::Char('l') | KeyCode::Right => editor.move_cursor(CursorMove::Right),
                        KeyCode::Char('H') | KeyCode::Char('^') => {
                            editor.move_cursor(CursorMove::LineBegin)
                        }
                        KeyCode::Char('L') | KeyCode::Char('$') => {
                            editor.move_cursor(CursorMove::LineEnd)
                        }
                        KeyCode::Char('w') => editor.move_cursor(CursorMove::WordStartForward),
                        KeyCode::Char('b') => editor.move_cursor(CursorMove::WordStartBackward),
                        KeyCode::Char('e') => editor.move_cursor(CursorMove::WordEndForward),
                        // TODO: This should be dd so find some sort of chord implementation
                        KeyCode::Char('i') => editor.mode = EditMode::Insert,
                        KeyCode::Char('I') => {
                            editor.move_cursor(CursorMove::LineBegin);
                            // TODO: maybe i should just make a editor.insert_mode() function
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
                        KeyCode::Char('d') => { editor.delete_line_at_cursor(); }
                        KeyCode::Char('s') => {
                            if key.modifiers.contains(KeyModifiers::CONTROL) {
                                editor.save()?;
                            }
                        }
                        KeyCode::Char('o') => editor.newline_under_cursor(),
                        KeyCode::Char('O') => editor.newline_above_cursor(),
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
                        _ => (),
                    }
                },
            }
        },
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
    let mut terminal = Terminal::new(backend)?;
    
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
