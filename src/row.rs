use crate::document::Tokenizer;
use crate::highlighting::{HighlightedText, TextMode};
use crate::{highlighting, Position, SearchDirection};
use std::cmp;
use std::cmp::Ordering;
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
    pub(crate) fn whitespace_pad(&mut self, total_len: usize) {
        let whitespace_len = total_len.saturating_sub(self.string.len());
        if whitespace_len > 0 {
            let whitespace = vec![" "; whitespace_len].join("");
            self.string.push_str(&whitespace);
            self.len = whitespace_len;
        }
    }

    pub(crate) fn token(&self, index: usize) -> Option<&Token> {
        self.tokens.get(index)
    }

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
            TextMode::Token => match self.tokens.len() {
                0 => usize::MAX,
                n => n.saturating_sub(1),
            },
            TextMode::Visual(Position { longest_row, .. }) => longest_row,
            TextMode::Search(_) => self.len.saturating_sub(1),
        }
    }

    pub(crate) fn find(&self, query: &str, at: usize, direction: SearchDirection) -> Option<usize> {
        if at > self.len || query.is_empty() {
            return None;
        }
        let (start, end) = match direction {
            SearchDirection::Forward => (at, self.len),
            SearchDirection::Backward => (0, at),
        };

        let substring: String = self.string.chars().skip(start).take(end - start).collect();
        match direction {
            SearchDirection::Forward => substring.find(query),
            SearchDirection::Backward => substring.rfind(query),
        }
        .map(|i| i + start)
    }

    pub(crate) fn highlight(&mut self, text: &HighlightedText) {
        self.highlighting = vec![highlighting::Type::None; self.string.len()];
        match text.mode {
            TextMode::Token => {
                if let Some(tok) = self.token(text.position.x) {
                    for i in tok.start..tok.start + tok.len {
                        if let Some(highlighting) = self.highlighting.get_mut(i) {
                            *highlighting = highlighting::Type::Highlighted;
                        };
                    }
                }
            }
            TextMode::Search(maybe_len) => {
                if let Some(len) = maybe_len {
                    for i in text.position.x..text.position.x + len {
                        if let Some(highlighting) = self.highlighting.get_mut(i) {
                            *highlighting = highlighting::Type::Highlighted;
                        };
                    }
                }
            }
            TextMode::Visual(start_pos) => {
                let (start, end) = switch_start_end(start_pos.x, text.position.x);
                for i in start..end + 1 {
                    if let Some(highlighting) = self.highlighting.get_mut(i) {
                        *highlighting = highlighting::Type::Highlighted;
                    };
                }
            }
        }
    }

    pub(crate) fn unhighlight(&mut self) {
        self.highlighting = vec![];
    }
}

pub(crate) fn switch_start_end(x1: usize, x2: usize) -> (usize, usize) {
    match x1.cmp(&x2) {
        Ordering::Less | Ordering::Equal => (x1, x2),
        Ordering::Greater => (x2, x1),
    }
}

#[cfg(test)]
#[path = "tests/test_row.rs"]
mod tests;
