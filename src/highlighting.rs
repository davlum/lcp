use crate::Position;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(crate) enum TextMode {
    Token,
    /// Visual contains a position which is the starting position of the highlighting
    Visual(Position),
    /// the str len is optional as there may be no matches to highlight
    Search(Option<usize>),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct HighlightedText {
    pub(crate) position: Position,
    pub(crate) mode: TextMode,
}

impl HighlightedText {
    pub(crate) fn new_search(position: Position, len: Option<usize>) -> Self {
        Self {
            position,
            mode: TextMode::Search(len),
        }
    }
    pub(crate) fn new_token(position: Position) -> Self {
        Self {
            position,
            mode: TextMode::Token,
        }
    }

    pub(crate) fn new_visual(position: Position) -> Self {
        Self {
            position,
            mode: TextMode::Visual(position),
        }
    }

    pub(crate) fn update_position(&mut self, position: Position) {
        self.position = position;
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Type {
    None,
    Highlighted,
}
