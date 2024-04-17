use std::collections::HashMap;

use anyhow::Result;
use lazy_static::lazy_static;

use crate::editor::{CurrentScreen, EditMode, Editor};

lazy_static! {
    pub static ref COMMAND_DICT: HashMap<&'static str, Command> = {
        use Command::*;
        
        let mut m = HashMap::new();
        m.insert("q", Quit);
        m.insert("q!", ForceQuit);
        m.insert("w", Write);
        m.insert("w!", ForceWrite);
        m.insert("wq", WriteAndQuit);
        m.insert("wq!", ForceWriteAndQuit);
        
        m
    };
}

    #[derive(Clone, Copy)]
pub enum Command {
    GotoLine,
    Quit,
    ForceQuit,
    Write,
    ForceWrite,
    WriteAndQuit,
    ForceWriteAndQuit,
}

impl Command {
    pub fn execute(self, editor: &mut Editor) -> Result<()> {
        match self {
            Command::Quit => {
                if editor.is_dirty()? {
                    editor.current_screen = CurrentScreen::Exiting;
                    return Ok(());
                }

                editor.running = false;
                Ok(())
            },
            Command::ForceQuit => {
                editor.running = false;
                Ok(())
            },
            Command::Write => editor.save(),
            Command::ForceWrite => editor.save(),
            Command::WriteAndQuit => {
                Command::Write.execute(editor)?;
                Command::Quit.execute(editor)?;
                Ok(())
            },
            Command::ForceWriteAndQuit => {
                Command::ForceWrite.execute(editor)?;
                Command::ForceQuit.execute(editor)?;
                Ok(())
            },
            Command::GotoLine => {
                if editor.command.parse::<u32>().is_ok() {
                    editor.cursor.y = editor.command.parse::<usize>()?.saturating_sub(1).clamp(0, editor.lines.len() - 1);
                }
                Ok(())
            },
        }
    }
}
