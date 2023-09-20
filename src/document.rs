use crate::Position;
use crate::Row;
use crate::SearchDirection;

use crate::highlighting::HighlightedText;
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
    pub(crate) fn open(input: impl BufRead, tokenizer: Tokenizer) -> Result<Self, std::io::Error> {
        let mut rows = Vec::new();
        for value in input.lines() {
            rows.push(Row::new(value?.as_str(), &tokenizer));
        }
        Ok(Self { rows })
    }
    pub(crate) fn row(&self, index: usize) -> Option<&Row> {
        self.rows.get(index)
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
    pub(crate) fn len(&self) -> usize {
        self.rows.len()
    }

    fn unhighlight_rows(&mut self) {
        for row in self.rows.iter_mut() {
            row.unhighlight();
        }
    }

    #[allow(clippy::indexing_slicing)]
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
                    position.x = self.rows[position.y].len();
                }
            } else {
                return None;
            }
        }
        None
    }

    pub(crate) fn get_text_at_pos(&self, position: &Position) -> Option<HighlightedText> {
        if let Some(row) = self.rows.get(position.y) {
            if let Some(tok) = row.tokens().get(position.x) {
                let text = row.get_slice(tok);
                let htext = HighlightedText {
                    text: text.to_string(),
                    token: *tok,
                    position: *position,
                };
                return Some(htext);
            }
        }
        None
    }

    pub(crate) fn highlight(
        &mut self,
        token_position: &Position,
        maybe_text: &Option<HighlightedText>,
    ) {
        self.unhighlight_rows();
        if let Some(text) = maybe_text {
            if let Some(row) = self.rows.get_mut(text.position.y) {
                row.highlight(text);
            }
        } else if let Some(row) = self.rows.get_mut(token_position.y) {
            if let Some(tok) = row.tokens().get(token_position.x) {
                let text = row.get_slice(tok);
                let htext = HighlightedText {
                    text: text.to_string(),
                    token: *tok,
                    position: *token_position,
                };
                row.highlight(&htext);
            }
        }
    }
}
