use crate::document::Tokenizer;
use crate::highlighting::{HighlightedText, TextMode};
use crate::{highlighting, SearchDirection};
use std::cmp;
use termion::color;
use unicode_segmentation::UnicodeSegmentation;

const HIGHLIGHTING_COLOR: color::LightWhite = color::LightWhite;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Token {
    pub(crate) start: usize,
    pub(crate) len: usize,
}

#[derive(Debug, Eq, PartialEq)]
pub struct Row {
    pub(crate) string: String,
    highlighting: Vec<highlighting::Type>,
    pub(crate) tokens: Vec<Token>,
    pub is_highlighted: bool,
    pub(crate) len: usize,
}

fn mk_tok_and_update_start(slice: &str, tok_s: &str, start: usize) -> (Token, usize) {
    let (divider, _) = slice[start..].split_once(tok_s).unwrap();
    let div_len = divider.len();
    let tok_len = tok_s.len();
    let tok = Token {
        start: start + div_len,
        len: tok_len,
    };
    (tok, start + div_len + tok_len)
}

pub(crate) fn mk_tokens(slice: &str, tokenizer: &Tokenizer) -> Vec<Token> {
    let mut tokens = vec![];
    let mut start = 0;
    match &tokenizer {
        Tokenizer::Whitespace => {
            for tok in slice.split_whitespace() {
                let (tok, new_start) = mk_tok_and_update_start(slice, tok, start);
                tokens.push(tok);
                start = new_start;
            }
        }
        Tokenizer::String(s) => {
            for tok in slice.split(s) {
                let (tok, new_start) = mk_tok_and_update_start(slice, tok, start);
                tokens.push(tok);
                start = new_start;
            }
        }
    }
    tokens
}

impl Row {
    pub(crate) fn new(slice: &str, tokenizer: &Tokenizer) -> Self {
        Self {
            string: String::from(slice),
            highlighting: Vec::new(),
            tokens: mk_tokens(slice, tokenizer),
            is_highlighted: false,
            len: slice.graphemes(true).count(),
        }
    }
}

impl Row {
    pub(crate) fn token(&self, index: usize) -> &Token {
        self.tokens
            .get(index)
            .unwrap_or_else(|| panic!("Should be a token at: {index}"))
    }

    // pub(crate) fn render(&self, start: usize, end: usize) -> String {
    //     let end = cmp::min(end, self.string.len());
    //     let start = cmp::min(start, end);
    //     let mut result = String::new();
    //     let mut current_highlighting = &highlighting::Type::None;
    //     #[allow(clippy::integer_arithmetic)]
    //     for (index, grapheme) in self.string[..]
    //         .graphemes(true)
    //         .enumerate()
    //         .skip(start)
    //         .take(end - start)
    //     {
    //         if let Some(c) = grapheme.chars().next() {
    //             let highlighting_type = self
    //                 .highlighting
    //                 .get(index)
    //                 .unwrap_or(&highlighting::Type::None);
    //             if highlighting_type != current_highlighting {
    //                 current_highlighting = highlighting_type;
    //                 let start_highlight = format!("{}", color::Bg(HIGHLIGHTING_COLOR));
    //                 result.push_str(&start_highlight);
    //             }
    //             if c == '\t' {
    //                 result.push(' ');
    //             } else {
    //                 result.push(c);
    //             }
    //         }
    //     }
    //     let end_highlight = format!("{}", color::Bg(color::Reset));
    //     result.push_str(&end_highlight);
    //     result
    // }

    pub(crate) fn render(&self, start: usize, end: usize) -> String {
        let end = cmp::min(end, self.string.len());
        let start = cmp::min(start, end);
        let mut result = String::new();
        let mut current_highlighting = &highlighting::Type::None;
        for (index, c) in self
            .string
            .chars()
            .enumerate()
            .skip(start)
            .take(end - start)
        {
            let highlighting_type = self
                .highlighting
                .get(index)
                .unwrap_or(&highlighting::Type::None);
            if highlighting_type != current_highlighting {
                current_highlighting = highlighting_type;
                let start_highlight = match highlighting_type {
                    highlighting::Type::None => format!("{}", color::Bg(color::Reset)),
                    highlighting::Type::Highlighted => format!("{}", color::Bg(HIGHLIGHTING_COLOR)),
                };
                result.push_str(&start_highlight);
            }
            if c == '\t' {
                result.push(' ');
            } else {
                result.push(c);
            }
        }
        let end_highlight = format!("{}", color::Bg(color::Reset));
        result.push_str(&end_highlight);
        result
    }

    pub(crate) fn len(&self, text_mode: TextMode) -> usize {
        match text_mode {
            TextMode::Token => self.tokens.len() - 1,
            TextMode::Cursor(_) => self.len,
        }
    }

    pub(crate) fn find(&self, query: &str, at: usize, direction: SearchDirection) -> Option<usize> {
        if at > self.len || query.is_empty() {
            return None;
        }
        let start = if direction == SearchDirection::Forward {
            at
        } else {
            0
        };
        let end = if direction == SearchDirection::Forward {
            self.len
        } else {
            at
        };
        #[allow(clippy::integer_arithmetic)]
        let substring: String = self.string[..]
            .graphemes(true)
            .skip(start)
            .take(end - start)
            .collect();
        let matching_byte_index = if direction == SearchDirection::Forward {
            substring.find(query)
        } else {
            substring.rfind(query)
        };
        if let Some(matching_byte_index) = matching_byte_index {
            for (grapheme_index, (byte_index, _)) in
                substring[..].grapheme_indices(true).enumerate()
            {
                if matching_byte_index == byte_index {
                    #[allow(clippy::integer_arithmetic)]
                    return Some(start + grapheme_index);
                }
            }
        }
        None
    }

    pub(crate) fn highlight(&mut self, text: &HighlightedText) {
        self.highlighting = vec![highlighting::Type::None; self.string.len()];
        match text.mode {
            TextMode::Token => {
                let tok = self
                    .tokens
                    .get(text.position.x)
                    .expect("Token should be here");
                for i in tok.start..tok.start + tok.len {
                    self.highlighting[i] = highlighting::Type::Highlighted;
                }
            }
            TextMode::Cursor(start_pos) => {
                for i in start_pos.x..text.position.x {
                    self.highlighting[i] = highlighting::Type::Highlighted;
                }
            }
        }
    }

    pub(crate) fn unhighlight(&mut self) {
        self.highlighting = vec![];
    }
}

#[cfg(test)]
#[path = "tests/test_row.rs"]
mod tests;
