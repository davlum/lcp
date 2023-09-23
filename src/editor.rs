use arboard::Clipboard;
use termion::color;
use termion::event::Key;

use crate::document::Tokenizer;
use crate::highlighting::HighlightedText;
use crate::Document;
use crate::Terminal;

const HELP_STRING: &str =
    "HELP: esc = quit | ENTER = copy | / = find | t = change tokenizer | w = whitespace (default) | v = visual mode";

const TOKENIZER_STRING: &str = "Enter text to change the tokenizer (default is whitespace): ";

const SEARCH_STRING: &str = "(ESC to cancel | Arrows to navigate): ";

const VISUAL_CURSOR_STRING: &str = "(v = start highlighting | ESC to cancel)";

const VISUAL_BLOCK_STRING: &str = "(ESC to cancel | ENTER to copy )";

const STATUS_FG_COLOR: color::Rgb = color::Rgb(63, 63, 63);
const STATUS_BG_COLOR: color::Rgb = color::Rgb(239, 239, 239);
// const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum SearchDirection {
    Forward,
    Backward,
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum VisualMode {
    Cursor,
    Block,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum InputMode {
    Normal,
    Tokenizer,
    Search(SearchDirection),
    Visual(VisualMode),
}

impl InputMode {
    fn as_str(&self) -> &str {
        match self {
            InputMode::Normal => "Token",
            InputMode::Tokenizer => "Tokenizer",
            InputMode::Search(_) => "Search",
            InputMode::Visual(VisualMode::Cursor) => "Visual (Cursor)",
            InputMode::Visual(VisualMode::Block) => "Visual (Block)",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum ShouldQuit {
    No,
    Ye(CopyStatus),
}

pub struct Editor {
    should_quit: ShouldQuit,
    clipboard: Option<Clipboard>,
    terminal: Terminal,
    cursor_position: Position,
    offset: Position,
    document: Document,
    status_message: String,
    highlighted_text: HighlightedText,
    input_mode: InputMode,
    prompt_input: String,
}

impl Editor {
    pub fn run(&mut self) -> std::io::Result<()> {
        loop {
            if let Err(error) = self.refresh_screen() {
                self.die(error)?;
            }
            if let ShouldQuit::Ye(_) = &self.should_quit {
                break;
            }
            let pressed_key = self.terminal.read_key()?;
            if let Err(error) = self.process_keypress(pressed_key) {
                self.die(error)?;
            }
        }
        self.terminal.cursor_show()?;
        Ok(())
    }
    pub fn new(
        document: Document,
        clipboard: Option<Clipboard>,
        terminal: Terminal,
    ) -> Result<Self, std::io::Error> {
        let highlighted_text = HighlightedText::new_token(Position::default());

        Ok(Self {
            should_quit: ShouldQuit::No,
            clipboard,
            terminal,
            document,
            cursor_position: Position::default(),
            offset: Position::default(),
            status_message: HELP_STRING.to_string(),
            highlighted_text,
            input_mode: InputMode::Normal,
            prompt_input: "".to_string(),
        })
    }

    fn draw(&mut self) -> std::io::Result<()> {
        self.document.highlight(&self.highlighted_text);
        self.draw_rows()?;
        self.draw_status_bar()?;
        self.draw_message_bar()?;
        // if let InputMode::Visual(VisualMode::Cursor) = self.input_mode {
        //     self.terminal.cursor_position(&self.cursor_position)?;
        //     self.terminal.cursor_show()?;
        // }
        Ok(())
    }

    fn refresh_screen(&mut self) -> Result<(), std::io::Error> {
        self.terminal.cursor_hide()?;
        // Cursors will move whether they are hidden or not.
        // They need to be reset on each loop or the position will affect
        // where we start outputting on the tty.
        self.terminal.cursor_position(&Position::default())?;
        if let ShouldQuit::Ye(copy_status) = &self.should_quit {
            self.terminal.clear_screen()?;
            match copy_status {
                CopyStatus::Noop => self.terminal.writeln("Copied Nothing.")?,
                CopyStatus::Success(s) => {
                    self.terminal.writeln("Copied:\r\n")?;
                    for line in s.lines() {
                        self.terminal.writeln(line)?;
                    }
                }
                CopyStatus::Error(e) => {
                    self.terminal
                        .writeln("Error when copying to clipboard:\r\n")?;
                    for line in e.lines() {
                        self.terminal.writeln(line)?;
                    }
                }
            };
        } else {
            self.draw()?;
        }
        self.terminal.flush()
    }

    fn normal_mode(&mut self) {
        self.input_mode = InputMode::Normal;
        self.prompt_input = "".to_string();
        self.token_cursor();
        self.highlighted_text = HighlightedText::new_token(self.cursor_position);
        self.status_message = HELP_STRING.to_string();
    }

    fn token_mode(&mut self) {
        self.input_mode = InputMode::Tokenizer;
        self.prompt_input = "".to_string();
        self.status_message = TOKENIZER_STRING.to_string();
    }

    fn search_mode(&mut self) {
        if let InputMode::Normal = self.input_mode {
            self.normal_cursor();
        }
        self.input_mode = InputMode::Search(SearchDirection::Forward);
        self.prompt_input = "".to_string();
        self.highlighted_text = HighlightedText::new_search(self.cursor_position, None);
        self.status_message = SEARCH_STRING.to_string();
    }

    fn visual_mode(&mut self) {
        self.prompt_input = "".to_string();
        if let InputMode::Visual(VisualMode::Cursor) = self.input_mode {
            self.input_mode = InputMode::Visual(VisualMode::Block);
            self.status_message = VISUAL_BLOCK_STRING.to_string();
        } else {
            if let InputMode::Normal = self.input_mode {
                self.normal_cursor();
            }
            self.input_mode = InputMode::Visual(VisualMode::Cursor);
            self.highlighted_text = HighlightedText::new_visual(self.cursor_position);
            self.status_message = VISUAL_CURSOR_STRING.to_string();
        }
    }

    fn process_keypress_tokenizer(&mut self, pressed_key: Key) {
        match pressed_key {
            Key::Backspace => {
                self.prompt_input
                    .truncate(self.prompt_input.len().saturating_sub(1));
                self.status_message = format!("{}{}", TOKENIZER_STRING, self.prompt_input)
            }
            Key::Char('\n') => {
                self.document
                    .update_tokenizer(Tokenizer::String(self.prompt_input.clone()));
                self.normal_mode();
            }
            Key::Char(c) => {
                if !c.is_control() {
                    self.prompt_input.push(c);
                }
                self.status_message = format!("{}{}", TOKENIZER_STRING, self.prompt_input)
            }
            Key::Esc => {
                self.prompt_input.truncate(0);
                self.normal_mode()
            }
            _ => (),
        }
    }

    fn process_keypress_search(&mut self, search_direction: SearchDirection, pressed_key: Key) {
        match pressed_key {
            Key::Backspace if !self.prompt_input.is_empty() => {
                self.prompt_input
                    .truncate(self.prompt_input.len().saturating_sub(1));
            }
            Key::Right | Key::Down => {
                if let InputMode::Search(ref mut direction) = self.input_mode {
                    *direction = SearchDirection::Forward;
                }
                self.move_cursor(Key::Right);
            }
            Key::Left | Key::Up => {
                if let InputMode::Search(ref mut direction) = self.input_mode {
                    *direction = SearchDirection::Backward;
                }
                self.move_cursor(Key::Left);
            }
            Key::Char('\n') => {
                self.copy_and_exit();
            }
            Key::Char(c) => {
                if !c.is_control() {
                    self.prompt_input.push(c);
                }
            }
            Key::Esc => {
                self.prompt_input.truncate(0);
                self.normal_mode();
                return;
            }
            _ => {}
        }
        self.status_message = format!("{}{}", SEARCH_STRING, self.prompt_input);
        let len =
            match self
                .document
                .find(&self.prompt_input, &self.cursor_position, search_direction)
            {
                None => None,
                Some(position) => {
                    self.cursor_position = position;
                    self.scroll();
                    Some(self.prompt_input.len())
                }
            };

        self.highlighted_text = HighlightedText::new_search(self.cursor_position, len);
    }

    fn process_keypress(&mut self, pressed_key: Key) -> Result<(), std::io::Error> {
        match &self.input_mode {
            InputMode::Normal => {
                self.process_keypress_normal(pressed_key)?;
            }
            InputMode::Tokenizer => {
                self.process_keypress_tokenizer(pressed_key);
            }
            InputMode::Search(search_direction) => {
                self.process_keypress_search(*search_direction, pressed_key);
            }
            InputMode::Visual(_) => {
                self.process_keypress_normal(pressed_key)?;
            }
        }
        Ok(())
    }

    fn copy_and_exit(&mut self) {
        let s = self.document.get_text(&self.highlighted_text);
        if s.is_empty() {
            self.should_quit = ShouldQuit::Ye(CopyStatus::Noop);
            return;
        }
        let copy_status = match self.clipboard.as_mut() {
            None => CopyStatus::Success(s.to_string()),
            Some(clipboard) => match clipboard.set_text(&s) {
                Ok(_) => CopyStatus::Success(s.to_string()),
                Err(e) => CopyStatus::Error(e.to_string()),
            },
        };
        self.should_quit = ShouldQuit::Ye(copy_status);
    }

    fn process_keypress_normal(&mut self, pressed_key: Key) -> Result<(), std::io::Error> {
        match pressed_key {
            Key::Esc => {
                if let InputMode::Visual(_) = self.input_mode {
                    self.normal_mode()
                } else {
                    self.should_quit = ShouldQuit::Ye(CopyStatus::Noop)
                }
            }
            Key::Char('/') => {
                self.search_mode();
                return Ok(());
            }
            Key::Char('\r') | Key::Char('\n') => self.copy_and_exit(),
            Key::Char('t') => self.token_mode(),
            Key::Char('v') => {
                self.visual_mode();
                return Ok(());
            }
            Key::Char('w') => self.document.update_tokenizer(Tokenizer::Whitespace),
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
        match self.input_mode {
            InputMode::Normal => {
                self.highlighted_text = HighlightedText::new_token(self.cursor_position)
            }
            InputMode::Visual(VisualMode::Cursor) => {
                self.highlighted_text = HighlightedText::new_visual(self.cursor_position)
            }
            InputMode::Visual(VisualMode::Block) => {
                self.highlighted_text.update_position(self.cursor_position);
            }
            _ => {}
        };
        // self.status_message = format!("{:?}", self.highlighted_text);
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
            "{}/{} lines. Mode: {}. Tokenizer: {}",
            self.cursor_position.y.saturating_add(1),
            self.document.len(),
            self.input_mode.as_str(),
            self.document.tokenizer().as_str()
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

    fn die(&mut self, e: std::io::Error) -> std::io::Result<()> {
        self.terminal.clear_screen()?;
        self.terminal.cursor_show()?;
        panic!("{e}");
    }

    fn token_cursor(&mut self) {
        let Position { x, y } = self.cursor_position;
        let row = self.document.row(y);
        if row.len > 0 {
            let percent_row = x as f64 / row.len as f64;
            let x = (percent_row * row.tokens.len() as f64).floor() as usize;
            self.cursor_position = Position { x, y };
        }
    }

    fn normal_cursor(&mut self) {
        let Position { x, y } = self.cursor_position;
        let row = self.document.row(y);
        let tok = row.token(x);
        self.cursor_position = Position { x: tok.start, y };
    }
}

#[cfg(test)]
#[path = "tests/test_editor.rs"]
mod tests;
