use crate::Position;

#[derive(Clone, Debug)]
pub(crate) struct HighlightedText {
    pub(crate) text: String,
    pub(crate) left_position: Position,
    pub(crate) right_position: Position,
}

impl HighlightedText {
    pub(crate) fn from_word(text: &str, position: Position) -> HighlightedText {
        let right_x = position.x + text.len();
        HighlightedText {
            text: text.to_string(),
            left_position: position,
            right_position: Position {
                x: right_x,
                y: position.y,
            },
        }
    }
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Type {
    None,
    Highlighted,
}
