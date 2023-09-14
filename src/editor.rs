use crate::highlighting::HighlightedText;
use crate::Document;
use crate::Row;
use crate::Terminal;
use arboard::Clipboard;
use std::fs::File;
use std::io::BufReader;
use std::{env, io};
use termion::color;
use termion::event::Key;

const STATUS_FG_COLOR: color::Rgb = color::Rgb(63, 63, 63);
const STATUS_BG_COLOR: color::Rgb = color::Rgb(239, 239, 239);
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum SearchDirection {
    Forward,
    Backward,
}

#[derive(Default, Clone, Copy, Debug)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

pub struct Editor {
    should_quit: bool,
    clipboard: Clipboard,
    terminal: Terminal,
    cursor_position: Position,
    offset: Position,
    document: Document,
    status_message: String,
    highlighted_text: Option<HighlightedText>,
}

impl Editor {
    pub fn run(&mut self) -> std::io::Result<()> {
        loop {
            if let Err(error) = self.refresh_screen() {
                self.die(error)?;
            }
            if self.should_quit {
                break;
            }
            if let Err(error) = self.process_keypress() {
                self.die(error)?;
            }
        }
        Ok(())
    }
    pub fn default() -> Result<Self, std::io::Error> {
        let args: Vec<String> = env::args().collect();
        let initial_status = String::from("HELP: Ctrl-F = find | Ctrl-S = save | Ctrl-Q = quit");

        let document = if let Some(file_name) = args.get(1) {
            let reader = BufReader::new(File::open(file_name)?);
            Document::open(reader)?
        } else {
            let stdin = io::stdin();
            let lines = BufReader::new(stdin.lock());
            Document::open(lines)?
        };

        Ok(Self {
            should_quit: false,
            clipboard: Clipboard::new().expect("Failed to initialize clipboard"),
            terminal: Terminal::default().expect("Failed to initialize terminal"),
            document,
            cursor_position: Position::default(),
            offset: Position::default(),
            status_message: initial_status,
            highlighted_text: None,
        })
    }

    fn refresh_screen(&mut self) -> Result<(), std::io::Error> {
        self.terminal.cursor_hide()?;
        self.terminal.cursor_position(&Position::default())?;
        if self.should_quit {
            self.terminal.clear_screen()?;
            match &self.highlighted_text {
                None => self.terminal.writeln("Copied Nothing.")?,
                Some(word) => self.terminal.writeln(&format!("Copied: {}", word.text))?,
            }
        } else {
            self.document.highlight(&self.highlighted_text);
            self.draw_rows()?;
            self.draw_status_bar()?;
            self.draw_message_bar()?;
            self.terminal.cursor_position(&Position {
                x: self.cursor_position.x.saturating_sub(self.offset.x),
                y: self.cursor_position.y.saturating_sub(self.offset.y),
            })?;
        }
        self.terminal.cursor_show()?;
        self.terminal.flush()
    }

    fn search(&mut self) {
        let old_position = self.cursor_position.clone();
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
                    let text = HighlightedText::from_word(query, editor.cursor_position);
                    editor.highlighted_text = Some(text);
                },
            )
            .unwrap_or(None);

        if query.is_none() {
            self.cursor_position = old_position;
            self.scroll();
        }
        self.highlighted_text = None;
    }
    fn process_keypress(&mut self) -> Result<(), std::io::Error> {
        let pressed_key = self.terminal.read_key()?;
        match pressed_key {
            Key::Ctrl('q' | 'c') | Key::Char('q') | Key::Esc => {
                self.highlighted_text = None;
                self.should_quit = true;
            }
            Key::Ctrl('f' | 's') | Key::Char('/') => self.search(),
            Key::Char('\r') | Key::Char('\n') => {
                if let Some(highlighted_word) = &self.highlighted_text {
                    self.clipboard
                        .set_text(highlighted_word.text.to_string())
                        .expect("Could not copy to clipboard");
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
            | Key::Home => self.move_cursor(pressed_key),
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
        let height = self.document.len();
        let mut width = if let Some(row) = self.document.row(y) {
            row.len()
        } else {
            0
        };
        match key {
            Key::Up => y = y.saturating_sub(1),
            Key::Down => {
                if y < height {
                    y = y.saturating_add(1);
                }
            }
            Key::Left => {
                if x > 0 {
                    x -= 1;
                } else if y > 0 {
                    y -= 1;
                    if let Some(row) = self.document.row(y) {
                        x = row.len();
                    } else {
                        x = 0;
                    }
                }
            }
            Key::Right => {
                if x < width {
                    x += 1;
                } else if y < height {
                    y += 1;
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
        width = if let Some(row) = self.document.row(y) {
            row.len()
        } else {
            0
        };
        if x > width {
            x = width;
        }

        self.cursor_position = Position { x, y }
    }
    fn draw_welcome_message(&self) -> std::io::Result<()> {
        let mut welcome_message = format!("Hecto editor -- version {VERSION}");
        let width = self.terminal.size().width as usize;
        let len = welcome_message.len();
        #[allow(clippy::integer_arithmetic, clippy::integer_division)]
        let padding = width.saturating_sub(len) / 2;
        let spaces = " ".repeat(padding.saturating_sub(1));
        welcome_message = format!("~{spaces}{welcome_message}");
        welcome_message.truncate(width);
        self.terminal.writeln(&welcome_message)
    }
    pub fn draw_row(&self, row: &Row) -> std::io::Result<()> {
        let width = self.terminal.size().width as usize;
        let start = self.offset.x;
        let end = self.offset.x.saturating_add(width);
        let row = row.render(start, end);
        self.terminal.writeln(&row)
    }
    #[allow(clippy::integer_division, clippy::integer_arithmetic)]
    fn draw_rows(&self) -> std::io::Result<()> {
        let height = self.terminal.size().height;
        for terminal_row in 0..height {
            self.terminal.clear_current_line()?;
            if let Some(row) = self
                .document
                .row(self.offset.y.saturating_add(terminal_row as usize))
            {
                self.draw_row(row)?;
            } else if self.document.is_empty() && terminal_row == height / 3 {
                self.draw_welcome_message()?;
            } else {
                self.terminal.writeln("~")?;
            }
        }
        Ok(())
    }
    fn draw_status_bar(&self) -> std::io::Result<()> {
        let mut status;
        let width = self.terminal.size().width as usize;

        status = format!("{} lines", self.document.len());

        let line_indicator = format!(
            "Line {}/{}",
            self.cursor_position.y.saturating_add(1),
            self.document.len()
        );

        let len = status.len() + line_indicator.len();
        status.push_str(&" ".repeat(width.saturating_sub(len)));
        status = format!("{status}{line_indicator}");
        status.truncate(width);
        self.terminal.set_bg_color(STATUS_BG_COLOR)?;
        self.terminal.set_fg_color(STATUS_FG_COLOR)?;
        self.terminal.writeln(&status.to_string())?;
        self.terminal.reset_fg_color()?;
        self.terminal.reset_bg_color()
    }

    fn draw_message_bar(&self) -> std::io::Result<()> {
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

    fn die(&self, e: std::io::Error) -> std::io::Result<()> {
        self.terminal.clear_screen()?;
        panic!("{e}");
    }
}
