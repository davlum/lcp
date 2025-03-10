use std::io::BufRead;

use crate::Position;
use crate::Row;
use crate::SearchDirection;
use crate::highlighting::{HighlightedText, TextMode};
use crate::row::{mk_tokens, switch_start_end};

#[derive(Clone, Debug)]
pub enum Tokenizer {
    Whitespace,
    String(String),
}

impl Tokenizer {
    pub(crate) fn as_str(&self) -> String {
        match self {
            Tokenizer::Whitespace => "whitespace (default)".to_string(),
            Tokenizer::String(s) => format!("'{s}'"),
        }
    }
}

#[derive(Debug)]
pub struct Document {
    rows: Vec<Row>,
    // In visual block mode, we consider the length of each row
    // to be equivalent to the longest row.
    longest_row: usize,
    tokenizer: Tokenizer,
}

impl Document {
    pub(crate) fn tokenizer(&self) -> &Tokenizer {
        &self.tokenizer
    }

    pub(crate) fn longest_row(&self) -> usize {
        self.longest_row
    }

    pub(crate) fn update_tokenizer(&mut self, tokenizer: Tokenizer) {
        for row in self.rows.iter_mut() {
            row.tokens = mk_tokens(&row.string, &tokenizer)
        }
        self.tokenizer = tokenizer;
    }
    pub(crate) fn new(input: impl BufRead) -> Result<Self, std::io::Error> {
        let tokenizer = Tokenizer::Whitespace;
        let mut rows = Vec::new();
        let mut longest_row = 0;

        for value in input.lines() {
            let row = Row::new(value?.as_str().trim_end(), &tokenizer);
            longest_row = std::cmp::max(longest_row, row.len);
            rows.push(row);
        }
        for row in rows.iter_mut() {
            row.whitespace_pad(longest_row);
        }

        Ok(Self {
            rows,
            tokenizer,
            longest_row: longest_row.saturating_sub(1),
        })
    }
    pub(crate) fn row(&self, index: usize) -> &Row {
        self.rows
            .get(index)
            .unwrap_or_else(|| panic!("Expected row at: {index}"))
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
    pub(crate) fn len(&self) -> usize {
        self.rows.len()
    }

    pub(crate) fn unhighlight_rows(&mut self) {
        for row in self.rows.iter_mut() {
            row.unhighlight();
        }
    }

    pub(crate) fn find(
        &self,
        query: &str,
        at: &Position,
        direction: SearchDirection,
    ) -> Option<Position> {
        if at.y >= self.rows.len() {
            return None;
        }
        let mut position = Position {
            x: at.x,
            y: at.y,
            longest_row: self.longest_row(),
        };

        for _ in 0..self.len() {
            if let Some(row) = self.rows.get(position.y) {
                if let Some(x) = row.find(query, position.x, direction) {
                    position.x = x;
                    return Some(position);
                }
                if direction == SearchDirection::Forward {
                    position.y = position.y.saturating_add(1);
                    if position.y == self.len() - 1 {
                        position.y = 0;
                    }
                    position.x = 0;
                } else {
                    position.y = position.y.saturating_sub(1);
                    if position.y == 0 {
                        position.y = self.len() - 1
                    }
                    position.x = self.rows[position.y].len;
                }
            } else {
                return None;
            }
        }
        None
    }

    pub(crate) fn highlight(&mut self, text: &HighlightedText) {
        self.unhighlight_rows();
        if let TextMode::Visual(start_position) = text.mode {
            let (start, end) = switch_start_end(start_position.y, text.position.y);
            for row_index in start..end + 1 {
                if let Some(row) = self.rows.get_mut(row_index) {
                    row.highlight(text)
                }
            }
        } else if let Some(row) = self.rows.get_mut(text.position.y) {
            row.highlight(text)
        }
    }

    pub(crate) fn get_text(&self, text: &HighlightedText) -> String {
        match text.mode {
            TextMode::Token => {
                let row = self.row(text.position.y);
                let token = match row.token(text.position.x) {
                    None => return "".to_string(),
                    Some(t) => t,
                };
                row.string[token.start..token.start + token.len].to_string()
            }
            TextMode::Visual(start_pos) => {
                if start_pos != text.position {
                    let (start, end) = switch_start_end(start_pos.y, text.position.y);
                    let mut lines = Vec::new();
                    for row_index in start..end + 1 {
                        let row = self.row(row_index);
                        let (start, end) = switch_start_end(start_pos.x, text.position.x);
                        // if end + 1 >= row.string.len() {
                        //     end = row.string.len();
                        // }
                        lines.push(&row.string[start..end + 1]);
                    }
                    lines.join("\n")
                } else {
                    String::new()
                }
            }
            TextMode::Search(Some(len)) => {
                let row = self.row(text.position.y);
                row.string[text.position.x..text.position.x + len].to_string()
            }
            TextMode::Search(None) => String::new(),
        }
    }
}

#[cfg(test)]
#[path = "tests/test_document.rs"]
mod tests;
