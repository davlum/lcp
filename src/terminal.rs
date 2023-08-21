use crate::Position;
use std::fs;
use std::io::{self, stdout, Write};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::{color, get_tty};

pub struct Size {
    pub width: u16,
    pub height: u16,
}
pub struct Terminal {
    size: Size,
    _stdout: RawTerminal<std::io::Stdout>,
    tty: fs::File,
}

impl Terminal {
    pub fn default() -> Result<Self, std::io::Error> {
        let size = termion::terminal_size()?;
        Ok(Self {
            size: Size {
                width: size.0,
                height: size.1.saturating_sub(2),
            },
            _stdout: stdout().into_raw_mode()?,
            tty: get_tty()?,
        })
    }
    pub fn size(&self) -> &Size {
        &self.size
    }
    pub fn clear_screen(&self) {
        write!(&self.tty, "{}", termion::clear::All);
    }

    #[allow(clippy::cast_possible_truncation)]
    pub fn cursor_position(&self, position: &Position) {
        let Position { mut x, mut y } = position;
        x = x.saturating_add(1);
        y = y.saturating_add(1);
        let x = x as u16;
        let y = y as u16;
        write!(&self.tty, "{}", termion::cursor::Goto(x, y));
    }
    pub fn flush(&self) -> Result<(), std::io::Error> {
        std::io::stdout().flush()
    }
    pub fn read_key(&self) -> Result<Key, std::io::Error> {
        loop {
            if let Some(key) = self.tty.try_clone()?.keys().next() {
                return key;
            }
        }
    }
    pub fn cursor_hide(&self) {
        write!(&self.tty, "{}", termion::cursor::Hide);
    }
    pub fn cursor_show(&self) {
        write!(&self.tty, "{}", termion::cursor::Show);
    }
    pub fn clear_current_line(&self) {
        write!(&self.tty, "{}", termion::clear::CurrentLine);
    }
    pub fn set_bg_color(&self, color: color::Rgb) {
        write!(&self.tty, "{}", color::Bg(color));
    }
    pub fn reset_bg_color(&self) {
        write!(&self.tty, "{}", color::Bg(color::Reset));
    }
    pub fn set_fg_color(&self, color: color::Rgb) {
        write!(&self.tty, "{}", color::Fg(color));
    }
    pub fn reset_fg_color(&self) {
        write!(&self.tty, "{}", color::Fg(color::Reset));
    }
}
