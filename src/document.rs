use crate::Position;
use crate::Row;
use crate::SearchDirection;

use crate::highlighting::{HighlightedText, TextMode};
use std::io::BufRead;

#[derive(Clone, Debug)]
pub enum Tokenizer {
    Whitespace,
    String(String),
}

#[derive(Debug)]
pub struct Document {
    rows: Vec<Row>,
}

impl Document {
    pub(crate) fn new(input: impl BufRead) -> Result<Self, std::io::Error> {
        let tokenizer = Tokenizer::Whitespace;
        let mut rows = Vec::new();
        for value in input.lines() {
            rows.push(Row::new(value?.as_str(), &tokenizer));
        }
        Ok(Self { rows })
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
        let mut position = Position { x: at.x, y: at.y };

        let start = if direction == SearchDirection::Forward {
            at.y
        } else {
            0
        };
        let end = if direction == SearchDirection::Forward {
            self.rows.len()
        } else {
            at.y.saturating_add(1)
        };
        for _ in start..end {
            if let Some(row) = self.rows.get(position.y) {
                if let Some(x) = row.find(query, position.x, direction) {
                    position.x = x;
                    return Some(position);
                }
                if direction == SearchDirection::Forward {
                    position.y = position.y.saturating_add(1);
                    position.x = 0;
                } else {
                    position.y = position.y.saturating_sub(1);
                    position.x = self.rows[position.y].len;
                }
            } else {
                return None;
            }
        }
        None
    }

    pub(crate) fn set_text(&self, text: &mut HighlightedText) {
        match text.mode {
            TextMode::Token => {
                let y = text.position.y;
                let x = text.position.x;
                let row = self.row(y);
                let tok = row.token(x);
                text.text = row.string[tok.start..tok.start + tok.len].to_string();
            }
            TextMode::Cursor(first_pos) => {
                let row = self.row(first_pos.y);
                text.text = row.string[first_pos.x..text.position.x].to_string();
            }
        };
    }

    pub(crate) fn highlight(&mut self, text: &HighlightedText) {
        self.unhighlight_rows();
        if let Some(row) = self.rows.get_mut(text.get_start_y()) {
            row.highlight(text)
        }
    }
}
