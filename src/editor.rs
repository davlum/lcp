use arboard::Clipboard;
use termion::color;
use termion::event::Key;

use crate::highlighting::HighlightedText;
use crate::Document;
use crate::Terminal;

const HELP_STRING: &str = "HELP: / = find | esc = quit | ENTER = copy highlighted text";

const STATUS_FG_COLOR: color::Rgb = color::Rgb(63, 63, 63);
const STATUS_BG_COLOR: color::Rgb = color::Rgb(239, 239, 239);
// const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum SearchDirection {
    Forward,
    Backward,
}

#[derive(Default, Clone, Copy, Debug, Eq, PartialEq)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum CopyStatus {
    Success(String),
    Error(String),
    Noop,
}

pub struct Editor {
    should_quit: bool,
    clipboard: Option<Clipboard>,
    terminal: Terminal,
    cursor_position: Position,
    offset: Position,
    document: Document,
    status_message: String,
    highlighted_text: HighlightedText,
    copy_status: CopyStatus,
}

impl Editor {
    pub fn run(&mut self) -> std::io::Result<()> {
        self.terminal.cursor_hide()?;
        loop {
            if let Err(error) = self.refresh_screen() {
                self.die(error)?;
            }
            if self.should_quit {
                break;
            }
            let pressed_key = self.terminal.read_key()?;
            if let Err(error) = self.process_keypress(pressed_key) {
                self.die(error)?;
            }
        }
        Ok(())
    }
    pub fn new(
        document: Document,
        clipboard: Option<Clipboard>,
        terminal: Terminal,
    ) -> Result<Self, std::io::Error> {
        let highlighted_text = HighlightedText::new_token(Position::default(), &document);

        Ok(Self {
            should_quit: false,
            clipboard,
            terminal,
            document,
            cursor_position: Position::default(),
            offset: Position::default(),
            status_message: HELP_STRING.to_string(),
            highlighted_text,
            copy_status: CopyStatus::Noop,
        })
    }

    fn refresh_screen(&mut self) -> Result<(), std::io::Error> {
        self.terminal.cursor_position(&Position::default())?;
        if self.should_quit {
            self.terminal.clear_screen()?;
            match &self.copy_status {
                CopyStatus::Noop => self.terminal.writeln("Copied Nothing.")?,
                CopyStatus::Success(s) => {
                    self.terminal.writeln(&format!("Copied:\r\n\r\n{}", s))?
                }
                CopyStatus::Error(e) => self
                    .terminal
                    .writeln(&format!("Error when copying to clipboard:\r\n\r\n{}", e))?,
            }
        } else {
            self.document.highlight(&self.highlighted_text);
            self.draw_rows()?;
            self.draw_status_bar()?;
            self.draw_message_bar()?;
        }
        self.terminal.flush()
    }

    fn search(&mut self) {
        let old_position = self.cursor_position;
        let mut direction = SearchDirection::Forward;
        let query = self
            .prompt(
                "Search (ESC to cancel, Arrows to navigate): ",
                |editor, key, query| {
                    let mut moved = false;
                    match key {
                        Key::Right | Key::Down => {
                            direction = SearchDirection::Forward;
                            editor.move_cursor(Key::Right);
                            moved = true;
                        }
                        Key::Left | Key::Up => direction = SearchDirection::Backward,
                        _ => direction = SearchDirection::Forward,
                    }
                    if let Some(position) =
                        editor
                            .document
                            .find(query, &editor.cursor_position, direction)
                    {
                        editor.cursor_position = position;
                        editor.scroll();
                    } else if moved {
                        editor.move_cursor(Key::Left);
                    }
                },
            )
            .unwrap_or(None);

        if query.is_none() {
            self.cursor_position = old_position;
            self.scroll();
        }
    }
    fn process_keypress(&mut self, pressed_key: Key) -> Result<(), std::io::Error> {
        match pressed_key {
            Key::Esc => {
                self.copy_status = CopyStatus::Noop;
                self.should_quit = true;
            }
            Key::Char('/') => {
                self.highlighted_text =
                    HighlightedText::new_cursor(self.cursor_position, &self.document);
                self.refresh_screen()?;
                self.search();
                self.highlighted_text =
                    HighlightedText::new_token(self.cursor_position, &self.document);
            }
            Key::Char('\r') | Key::Char('\n') => {
                let s = self.highlighted_text.text.to_string();
                match self.clipboard.as_mut() {
                    None => self.copy_status = CopyStatus::Success(s),
                    Some(clipboard) => match clipboard.set_text(s.clone()) {
                        Ok(_) => self.copy_status = CopyStatus::Success(s),
                        Err(e) => self.copy_status = CopyStatus::Error(e.to_string()),
                    },
                }

                self.should_quit = true;
            }
            Key::Up
            | Key::Down
            | Key::Left
            | Key::Right
            | Key::PageUp
            | Key::PageDown
            | Key::End
            | Key::Home
            | Key::Char('$' | '^') => self.move_cursor(pressed_key),
            _ => (),
        }
        self.scroll();
        Ok(())
    }
    fn scroll(&mut self) {
        let Position { x, y } = self.cursor_position;
        let width = self.terminal.size().width as usize;
        let height = self.terminal.size().height as usize;
        let mut offset = &mut self.offset;
        if y < offset.y {
            offset.y = y;
        } else if y >= offset.y.saturating_add(height) {
            offset.y = y.saturating_sub(height).saturating_add(1);
        }
        if x < offset.x {
            offset.x = x;
        } else if x >= offset.x.saturating_add(width) {
            offset.x = x.saturating_sub(width).saturating_add(1);
        }
    }
    fn move_cursor(&mut self, key: Key) {
        let terminal_height = self.terminal.size().height as usize;
        let Position { mut y, mut x } = self.cursor_position;
        let height = self.document.len() - 1;
        let row = self.document.row(y);
        let mut width = row.len(self.highlighted_text.mode);
        match key {
            Key::Char('$') => {
                x = width;
            }
            Key::Char('^') => {
                x = 0;
            }
            Key::Up => {
                if y == 0 {
                    y = height;
                } else {
                    y = y.saturating_sub(1)
                }
            }
            Key::Down => {
                if y < height {
                    y = y.saturating_add(1);
                } else {
                    y = 0;
                }
            }
            Key::Left => {
                if x > 0 {
                    x -= 1;
                } else {
                    x = width
                }
            }
            Key::Right => {
                if x < width {
                    x += 1;
                } else {
                    x = 0;
                }
            }
            Key::PageUp => {
                y = if y > terminal_height {
                    y.saturating_sub(terminal_height)
                } else {
                    0
                }
            }
            Key::PageDown => {
                y = if y.saturating_add(terminal_height) < height {
                    y.saturating_add(terminal_height)
                } else {
                    height
                }
            }
            Key::Home => x = 0,
            Key::End => x = width,
            _ => (),
        }
        let row = self.document.row(y);
        width = row.len(self.highlighted_text.mode);

        if x > width {
            x = width;
        }

        self.cursor_position = Position { x, y };
        self.update_highlighted_text();
    }

    pub fn draw_row(&mut self, index: usize) -> std::io::Result<()> {
        let row = self.document.row(index);
        let width = self.terminal.size().width as usize;
        let start = self.offset.x;
        let end = self.offset.x.saturating_add(width);
        let row = row.render(start, end);
        self.terminal.writeln(&row)
    }

    fn draw_rows(&mut self) -> std::io::Result<()> {
        let height = self.terminal.size().height;
        let doc_len = self.document.len() as u16;
        for terminal_row in 0..height {
            self.terminal.clear_current_line()?;
            if terminal_row < doc_len {
                self.draw_row(self.offset.y.saturating_add(terminal_row as usize))?;
            } else {
                self.terminal.writeln("~")?;
            }
        }
        Ok(())
    }
    fn draw_status_bar(&mut self) -> std::io::Result<()> {
        let width = self.terminal.size().width as usize;

        let mut line_indicator = format!(
            "{}/{} lines",
            self.cursor_position.y.saturating_add(1),
            self.document.len()
        );

        let len = line_indicator.len();
        line_indicator.push_str(&" ".repeat(width.saturating_sub(len)));
        line_indicator.truncate(width);
        self.terminal.set_bg_color(STATUS_BG_COLOR)?;
        self.terminal.set_fg_color(STATUS_FG_COLOR)?;
        self.terminal.writeln(&line_indicator.to_string())?;
        self.terminal.reset_fg_color()?;
        self.terminal.reset_bg_color()
    }

    fn draw_message_bar(&mut self) -> std::io::Result<()> {
        self.terminal.clear_current_line()?;
        let mut message = self.status_message.clone();
        message.truncate(self.terminal.size().width as usize);
        self.terminal.write(&message)?;
        Ok(())
    }
    fn prompt<C>(&mut self, prompt: &str, mut callback: C) -> Result<Option<String>, std::io::Error>
    where
        C: FnMut(&mut Self, Key, &String),
    {
        let mut result = String::new();
        loop {
            self.status_message = format!("{prompt}{result}");
            self.refresh_screen()?;
            let key = self.terminal.read_key()?;
            match key {
                Key::Backspace => result.truncate(result.len().saturating_sub(1)),
                Key::Char('\n') => break,
                Key::Char(c) => {
                    if !c.is_control() {
                        result.push(c);
                    }
                }
                Key::Esc => {
                    result.truncate(0);
                    break;
                }
                _ => (),
            }
            callback(self, key, &result);
        }
        self.status_message = String::new();
        if result.is_empty() {
            return Ok(None);
        }
        Ok(Some(result))
    }

    pub(crate) fn update_highlighted_text(&mut self) {
        self.highlighted_text
            .update_position(self.cursor_position, &self.document)
    }

    fn die(&mut self, e: std::io::Error) -> std::io::Result<()> {
        self.terminal.clear_screen()?;
        panic!("{e}");
    }
}

#[cfg(test)]
#[path = "tests/test_editor.rs"]
mod tests;
