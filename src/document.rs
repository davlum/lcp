use crate::Position;
use crate::Row;
use crate::SearchDirection;

use crate::highlighting::HighlightedText;
use std::io::BufRead;

#[derive(Default)]
pub struct Document {
    rows: Vec<Row>,
}

impl Document {
    pub(crate) fn open(input: impl BufRead) -> Result<Self, std::io::Error> {
        let mut rows = Vec::new();
        for value in input.lines() {
            rows.push(Row::from(value?.as_str()));
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

    fn unhighlight_rows(&mut self, start: usize) {
        let start = start.saturating_sub(1);
        for row in self.rows.iter_mut().skip(start) {
            row.is_highlighted = false;
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
    pub(crate) fn highlight(&mut self, maybe_text: &Option<HighlightedText>) {
        if let Some(text) = maybe_text {
            if let Some(row) = self.rows.get_mut(text.left_position.y) {
                row.highlight(text);
            }
        }
    }
}
