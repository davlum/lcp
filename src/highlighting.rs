use crate::{Document, Position};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(crate) enum TextMode {
    Token,
    /// Cursor contains a position which is the starting position of the highlighting
    Cursor(Position),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct HighlightedText {
    pub(crate) position: Position,
    pub(crate) text: String,
    pub(crate) mode: TextMode,
}

impl HighlightedText {
    pub(crate) fn new_cursor(position: Position, doc: &Document) -> Self {
        let mode = TextMode::Cursor(position);
        let mut htext = Self {
            position,
            text: "".to_string(),
            mode,
        };
        doc.set_text(&mut htext);
        htext
    }
    pub(crate) fn new_token(position: Position, doc: &Document) -> Self {
        let mut htext = Self {
            position,
            text: "".to_string(),
            mode: TextMode::Token,
        };
        doc.set_text(&mut htext);
        htext
    }

    pub(crate) fn update_position(&mut self, position: Position, doc: &Document) {
        self.position = position;
        doc.set_text(self);
    }

    pub(crate) fn get_start_y(&self) -> usize {
        match self.mode {
            TextMode::Token => self.position.y,
            TextMode::Cursor(pos) => pos.y,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Type {
    None,
    Highlighted,
}
