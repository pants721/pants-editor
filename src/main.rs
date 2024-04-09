use std::{
    io::{self, stdout},
    path::PathBuf,
};

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use editor::Editor;
use itertools::Itertools;
use ratatui::prelude::*;
use ui::ui;

mod cursor;
mod editor;
mod ui;
mod word;

fn main() -> Result<()> {
    enable_raw_mode()?;
    let mut stderr = io::stderr();
    execute!(stderr, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;

    let args = std::env::args();

    let mut editor = Editor::new();
    if args.len() > 1 {
        editor.open_file(&args.collect_vec()[1])?;
    } else {
        editor.open_file("src/editor.rs")?;
    }
    run_editor(&mut terminal, &mut editor)?;

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture,
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn run_editor<B: Backend>(terminal: &mut Terminal<B>, editor: &mut Editor) -> Result<bool> {
    loop {
        terminal.draw(|f| {
            ui(f, editor);
        })?;
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Release {
                continue;
            }

            if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('q') {
                return Ok(true);
            }

            editor.input(key);
        }
    }
}
