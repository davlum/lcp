use crate::document::Tokenizer;
use crate::highlighting;
use crate::highlighting::{HighlightedText, Token, Type};
use crate::SearchDirection;
use std::cmp;
use termion::color;
use unicode_segmentation::UnicodeSegmentation;

const HIGHLIGHTING_COLOR: color::LightWhite = color::LightWhite;

#[derive(Debug, Eq, PartialEq)]
pub enum RowMode {
    Token,
    String,
}

#[derive(Debug, Eq, PartialEq)]
pub struct Row {
    mode: RowMode,
    string: String,
    highlighting: Vec<highlighting::Type>,
    tokens: Vec<Token>,
    pub is_highlighted: bool,
    len: usize,
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

impl Row {
    pub(crate) fn new(slice: &str, tokenizer: &Tokenizer) -> Self {
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

        Self {
            mode: RowMode::Token,
            string: String::from(slice),
            highlighting: Vec::new(),
            tokens,
            is_highlighted: false,
            len: slice.graphemes(true).count(),
        }
    }
}

impl Row {
    pub(crate) fn tokens(&self) -> &Vec<Token> {
        &self.tokens
    }

    pub(crate) fn get_slice(&self, tok: &Token) -> &str {
        &self.string[tok.start..tok.start + tok.len]
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
                    Type::None => format!("{}", color::Bg(color::Reset)),
                    Type::Highlighted => format!("{}", color::Bg(HIGHLIGHTING_COLOR)),
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

    pub(crate) fn len(&self) -> usize {
        match self.mode {
            RowMode::Token => self.tokens.len() - 1,
            RowMode::String => self.len,
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
        let end = text.token.start + text.token.len;
        for i in text.token.start..end {
            self.highlighting[i] = highlighting::Type::Highlighted;
        }
    }

    pub(crate) fn unhighlight(&mut self) {
        self.highlighting = vec![];
    }
}

#[cfg(test)]
#[path = "tests/test_row.rs"]
mod tests;
