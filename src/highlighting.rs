use crate::Position;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(crate) enum TextMode {
    Token,
    /// Visual contains a position which is the starting position of the highlighting
    Visual(Position),
    Search(usize),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct HighlightedText {
    pub(crate) position: Position,
    pub(crate) mode: TextMode,
}

impl HighlightedText {
    pub(crate) fn new_search(position: Position, len: usize) -> Self {
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

    pub(crate) fn get_start_y(&self) -> usize {
        match self.mode {
            TextMode::Token | TextMode::Search(_) => self.position.y,
            TextMode::Visual(pos) => pos.y,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Type {
    None,
    Highlighted,
}
