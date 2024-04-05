use std::io::{self, stdout};

use anyhow::Result;
use editor::Editor;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers}, execute, terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}, ExecutableCommand
};
use ratatui::{prelude::*, widgets::*};
use ui::ui;

mod editor;
mod ui;

fn main() -> Result<()> {
    enable_raw_mode()?;
    let mut stderr = io::stderr();
    execute!(stderr, EnterAlternateScreen, EnableMouseCapture)?;
    
    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;

    let mut editor = Editor::new();
    editor.lines = vec![
        "line 1".to_string(),
        "line 2".to_string(),
        "line 3".to_string(),
        "line 4".to_string(),
        "".to_string(),
        "line 5".to_string(),
    ];
    let res = run_editor(&mut terminal, &mut editor);

    
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture,
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn run_editor<B: Backend>(
    terminal: &mut Terminal<B>,
    editor: &mut Editor,
) -> Result<bool> {
    loop {
        terminal.draw(|f| { 
            ui(f, editor);
        })?;
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Release {
                continue;
            }

            if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                return Ok(true);
            }

            editor.input(key);
        }
    } 
}

