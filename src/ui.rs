use ratatui::{
    prelude::*,
    widgets::{Paragraph, Wrap},
};

use crate::editor::Editor;

pub fn ui(f: &mut Frame, editor: &mut Editor) {
    let mut buffer_rect = f.size();
    buffer_rect.height = f.size().height - 2;

    if (editor.cursor.1 as u16) < editor.scroll.0 {
        editor.scroll.0 = editor.cursor.1 as u16;
    } else if (editor.cursor.1 as u16) >= editor.scroll.0 + buffer_rect.height {
        editor.scroll.0 = (editor.cursor.1 as u16) - buffer_rect.height.saturating_sub(1);
    }

    let line_number_width = (editor.scroll.0 + buffer_rect.height)
        .min(editor.lines.len() as u16)
        .to_string()
        .len() as u16
        + 1;

    buffer_rect.width = f.size().width - line_number_width;
    buffer_rect.x += line_number_width;

    let line_numbers = editor
        .lines
        .iter()
        .enumerate()
        .skip(editor.scroll.0 as usize)
        .take(buffer_rect.height as usize)
        .map(|(i, _)| format!("{}", i + 1));
    let line_numbers = line_numbers.collect::<Vec<String>>().join("\n");
    let line_numbers = Paragraph::new(line_numbers)
        .wrap(Wrap { trim: false })
        .dark_gray();
    let mut line_numbers_rect = f.size();
    line_numbers_rect.width = line_number_width;
    line_numbers_rect.height = buffer_rect.height;
    f.render_widget(line_numbers, line_numbers_rect);

    f.set_cursor(
        editor.cursor.0 as u16 + (f.size().width - buffer_rect.width),
        editor.cursor.1 as u16 - editor.scroll.0,
    );

    let buffer_block = Paragraph::new(editor.lines.join("\n")).scroll(editor.scroll);

    let statusline_block = Paragraph::new(format!(
        "[{}] {}:{}",
        editor.mode, editor.cursor.0, editor.cursor.1
    ))
    .on_red()
    .black();
    let mut statusline_rect = f.size();
    statusline_rect.height = 1;
    statusline_rect.y = f.size().height - 2;

    f.render_widget(buffer_block, buffer_rect);
    f.render_widget(statusline_block, statusline_rect);
}
