use crate::Position;
use std::fs;
use std::io::{stdout, Write};
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
    pub(crate) fn default() -> Result<Self, std::io::Error> {
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
    pub(crate) fn size(&self) -> &Size {
        &self.size
    }
    pub(crate) fn clear_screen(&self) -> std::io::Result<()> {
        write!(&self.tty, "{}", termion::clear::All)
    }

    pub(crate) fn cursor_position(&self, position: &Position) -> std::io::Result<()> {
        let Position { mut x, mut y } = position;
        x = x.saturating_add(1);
        y = y.saturating_add(1);
        let x = x as u16;
        let y = y as u16;
        write!(&self.tty, "{}", termion::cursor::Goto(x, y))
    }
    pub(crate) fn flush(&self) -> Result<(), std::io::Error> {
        std::io::stdout().flush()
    }
    pub(crate) fn read_key(&self) -> Result<Key, std::io::Error> {
        loop {
            if let Some(key) = self.tty.try_clone()?.keys().next() {
                return key;
            }
        }
    }
    pub(crate) fn writeln(&self, s: &str) -> std::io::Result<()> {
        writeln!(&self.tty, "{s}\r")
    }
    pub(crate) fn write(&self, s: &str) -> std::io::Result<()> {
        write!(&self.tty, "{s}")
    }
    pub(crate) fn cursor_hide(&self) -> std::io::Result<()> {
        write!(&self.tty, "{}", termion::cursor::Hide)
    }

    // pub(crate) fn cursor_show(&self) -> std::io::Result<()> {
    //     write!(&self.tty, "{}", termion::cursor::Show)
    // }
    pub(crate) fn clear_current_line(&self) -> std::io::Result<()> {
        write!(&self.tty, "{}", termion::clear::CurrentLine)
    }
    pub(crate) fn set_bg_color(&self, color: color::Rgb) -> std::io::Result<()> {
        write!(&self.tty, "{}", color::Bg(color))
    }
    pub(crate) fn reset_bg_color(&self) -> std::io::Result<()> {
        write!(&self.tty, "{}", color::Bg(color::Reset))
    }
    pub(crate) fn set_fg_color(&self, color: color::Rgb) -> std::io::Result<()> {
        write!(&self.tty, "{}", color::Fg(color))
    }
    pub(crate) fn reset_fg_color(&self) -> std::io::Result<()> {
        write!(&self.tty, "{}", color::Fg(color::Reset))
    }
}
