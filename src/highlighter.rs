
use anyhow::Result;
use itertools::Itertools;
use syntect::{highlighting::{HighlightIterator, HighlightState, Highlighter, Style, Theme}, parsing::{ParseState, ScopeStack, SyntaxReference}};

use crate::{SYNTAX_SET, THEME_SET};

pub struct SyntaxHighlighter<'a> {
    pub parse_state: ParseState,
    pub highlight_state: HighlightState,
    pub highlighter: Highlighter<'a>,
}

impl Default for SyntaxHighlighter<'_> {
    fn default() -> Self {
        let theme = &THEME_SET.themes["base16-mocha.dark"];
        let syntax = SYNTAX_SET.find_syntax_by_extension("rs").unwrap();
        Self::new(syntax.clone(), theme)
    }
}

impl<'a> SyntaxHighlighter<'a> {
    pub fn new(syntax: SyntaxReference, theme: &'a Theme) -> Self {
        let parse_state = ParseState::new(&syntax);
        let highlighter = Highlighter::new(theme);
        let highlight_state = HighlightState::new(&highlighter, ScopeStack::new());

        Self {
            parse_state,
            highlight_state,
            highlighter,
        }
    }

    pub fn initial_parse(&mut self, lines: Vec<String>) -> Result<()> {
        for line in lines {
            let ops = self.parse_state.parse_line(&line, &SYNTAX_SET)?;
            for (_, op) in ops {
                self.highlight_state.path.apply(&op)?;
            }
        }

        Ok(())
    }

    // pub fn get_closest_cache(&self, idx: usize) -> (ParseState, HighlightState) {
    //     let mut parse_state = self.parse_state.clone();
    //     let mut highlight_state = self.highlight_state.clone();
    //
    //     for (cache_idx, state) in &self.parse_state_cache {
    //         if *cache_idx < idx {
    //             parse_state = state.clone();
    //         }
    //     }
    //
    //     for (cache_idx, state) in &self.highlight_state_cache {
    //         if *cache_idx < idx {
    //             highlight_state = state.clone();
    //         }
    //     }
    //
    //     (parse_state, highlight_state)
    // }


    pub fn highlight_line(&mut self, line: &str) -> Result<Vec<(Style, String)>> {
        let ops = self.parse_state.parse_line(line, &SYNTAX_SET)?;
        let iter =
            HighlightIterator::new(&mut self.highlight_state, &ops[..], line, &self.highlighter);
        
        Ok(iter.map(|(st, s)| (st, s.to_string())).collect_vec())
    }
}

pub fn syntect_style_to_ratatui(style: &Style) -> ratatui::prelude::Style {
    let fg = style.foreground;
    let bg = style.background;
    ratatui::prelude::Style::default()
        .fg(ratatui::style::Color::Rgb(fg.r, fg.g, fg.b))
        .bg(ratatui::style::Color::Rgb(bg.r, bg.g, bg.b))
}
