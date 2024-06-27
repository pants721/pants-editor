use std::{
    io::{self, stdout},
    panic::{set_hook, take_hook},
};

use anyhow::{Context, Result};
use cli::Cli;
use config::Settings;
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use editor::{CurrentScreen, CursorMove, Editor};
use figment::{
    providers::{Format, Toml},
    Figment,
};
use lazy_static::lazy_static;
use ratatui::prelude::*;
use clap::Parser;
use syntect::{highlighting::ThemeSet, parsing::SyntaxSet};
use ui::ui;
use util::pe_config_file_path;

mod cli;
mod config;
mod cursor;
mod editor;
mod search;
mod ui;
mod util;
mod word;

lazy_static! {
    pub static ref SYNTAX_SET: SyntaxSet = SyntaxSet::load_defaults_newlines();
    pub static ref THEME_SET: ThemeSet = ThemeSet::load_defaults();
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let config_path = match cli.config {
        Some(cfg) => cfg,
        None => pe_config_file_path()?,
    };
    
    let config: Settings = Figment::new()
        .merge(Toml::file(config_path))
        .extract()
        .context("Failed to load config")?;
    let mut editor = Editor::new();
    editor.settings = config;

    if let Some(file) = cli.file {
        editor.open(&file)?;
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
            match key {
                KeyEvent {
                    code: KeyCode::Char('q'),
                    modifiers: KeyModifiers::ALT,
                    ..
                } => editor.current_screen = CurrentScreen::Exiting,
                
                KeyEvent {
                    code: KeyCode::Char('k'),
                    modifiers: KeyModifiers::ALT,
                    ..
                } | KeyEvent {
                    code: KeyCode::Up,
                    ..
                } => editor.move_cursor(CursorMove::Up),
                
                KeyEvent {
                    code: KeyCode::Char('j'),
                    modifiers: KeyModifiers::ALT,
                    ..
                } | KeyEvent {
                    code: KeyCode::Down,
                    ..
                } => editor.move_cursor(CursorMove::Down),
                
                KeyEvent {
                    code: KeyCode::Char('h'),
                    modifiers: KeyModifiers::ALT,
                    ..
                } | KeyEvent {
                    code: KeyCode::Left,
                    ..
                } => editor.move_cursor(CursorMove::Left),
                
                KeyEvent {
                    code: KeyCode::Char('l'),
                    modifiers: KeyModifiers::ALT,
                    ..
                } | KeyEvent {
                    code: KeyCode::Right,
                    ..
                } => editor.move_cursor(CursorMove::Right),

                KeyEvent {
                    code: KeyCode::Char('w'),
                    modifiers: KeyModifiers::ALT,
                    ..
                } => editor.move_cursor(CursorMove::WordStartForward),
                
                KeyEvent {
                    code: KeyCode::Char('b'),
                    modifiers: KeyModifiers::ALT,
                    ..
                } => editor.move_cursor(CursorMove::WordStartBackward),
                
                KeyEvent {
                    code: KeyCode::Char('e'),
                    modifiers: KeyModifiers::ALT,
                    ..
                } => editor.move_cursor(CursorMove::WordEndForward),
                
                KeyEvent {
                    code: KeyCode::Char('H'),
                    modifiers: KeyModifiers::ALT,
                    ..
                } => editor.move_cursor(CursorMove::LineBegin),
                
                KeyEvent {
                    code: KeyCode::Char('L'),
                    modifiers: KeyModifiers::ALT,
                    ..
                } => editor.move_cursor(CursorMove::LineEnd),
                
                KeyEvent { code: KeyCode::Enter, .. } => editor.newline_at_cursor(),
                KeyEvent { code: KeyCode::Backspace, .. } => editor.backspace_at_cursor(),
                KeyEvent { code: KeyCode::Tab, .. } => editor.insert_tab(),
                KeyEvent { code: KeyCode::Char(val), .. } => editor.insert_char_at_cursor(val),
                _ => (),
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
