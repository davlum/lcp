use crate::Position;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Token {
    pub(crate) start: usize,
    pub(crate) len: usize,
}

#[derive(Clone, Debug)]
pub(crate) struct HighlightedText {
    pub(crate) text: String,
    pub(crate) token: Token,
    pub(crate) position: Position,
}

impl HighlightedText {
    pub(crate) fn from_word(text: &str, position: Position) -> HighlightedText {
        HighlightedText {
            text: text.to_string(),
            position,
            token: Token {
                start: position.x,
                len: text.len(),
            },
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Type {
    None,
    Highlighted,
}
