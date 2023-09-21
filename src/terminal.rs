use std::fs;
use std::io::Write;

use crate::Position;
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
    _stdout: Option<RawTerminal<std::io::Stdout>>,
    tty: Option<fs::File>,
}

impl Terminal {
    pub(crate) fn new(maybe_size: Option<(u16, u16)>) -> Result<Self, std::io::Error> {
        let size = match maybe_size {
            None => termion::terminal_size()?,
            Some(tup) => tup,
        };

        let tty = match maybe_size {
            None => {
                let tty = get_tty()?;
                // Reset the cursor to the top corner of the screen
                write!(&tty, "{}", termion::cursor::Goto(1, 1))?;
                Some(tty)
            }
            Some(_) => None,
        };

        Ok(Self {
            size: Size {
                width: size.0,
                height: size.1.saturating_sub(2),
            },
            _stdout: if maybe_size.is_none() {
                Some(std::io::stdout().into_raw_mode()?)
            } else {
                None
            },
            tty,
        })
    }
    pub(crate) fn size(&self) -> &Size {
        &self.size
    }
    pub(crate) fn clear_screen(&mut self) -> std::io::Result<()> {
        if let Some(tty) = self.tty.as_mut() {
            write!(tty, "{}", termion::clear::All)?;
        }
        Ok(())
    }

    pub(crate) fn flush(&mut self) -> Result<(), std::io::Error> {
        if let Some(_) = self.tty.as_mut() {
            std::io::stdout().flush()?;
        }
        Ok(())
    }
    pub(crate) fn read_key(&mut self) -> Result<Key, std::io::Error> {
        if let Some(tty) = self.tty.as_mut() {
            loop {
                if let Some(key) = tty.try_clone()?.keys().next() {
                    return key;
                }
            }
        }
        // This only happens in test
        Ok(Key::Esc)
    }
    pub(crate) fn writeln(&mut self, s: &str) -> std::io::Result<()> {
        if let Some(tty) = self.tty.as_mut() {
            writeln!(tty, "{s}\r")?;
        }
        Ok(())
    }
    pub(crate) fn write(&mut self, s: &str) -> std::io::Result<()> {
        if let Some(tty) = self.tty.as_mut() {
            write!(tty, "{s}")?;
        }
        Ok(())
    }

    pub(crate) fn cursor_position(&mut self, position: &Position) -> std::io::Result<()> {
        if let Some(tty) = self.tty.as_mut() {
            let Position { mut x, mut y } = position;
            x = x.saturating_add(1);
            y = y.saturating_add(1);
            let x = x as u16;
            let y = y as u16;
            write!(tty, "{}", termion::cursor::Goto(x, y))?;
        }
        Ok(())
    }

    pub(crate) fn cursor_hide(&mut self) -> std::io::Result<()> {
        if let Some(tty) = self.tty.as_mut() {
            write!(tty, "{}", termion::cursor::Hide)?;
        }
        Ok(())
    }

    pub(crate) fn clear_current_line(&mut self) -> std::io::Result<()> {
        if let Some(tty) = self.tty.as_mut() {
            write!(tty, "{}", termion::clear::CurrentLine)?;
        }
        Ok(())
    }
    pub(crate) fn set_bg_color(&mut self, color: color::Rgb) -> std::io::Result<()> {
        if let Some(tty) = self.tty.as_mut() {
            write!(tty, "{}", color::Bg(color))?;
        }
        Ok(())
    }
    pub(crate) fn reset_bg_color(&mut self) -> std::io::Result<()> {
        if let Some(tty) = self.tty.as_mut() {
            write!(tty, "{}", color::Bg(color::Reset))?;
        }
        Ok(())
    }
    pub(crate) fn set_fg_color(&mut self, color: color::Rgb) -> std::io::Result<()> {
        if let Some(tty) = self.tty.as_mut() {
            write!(tty, "{}", color::Fg(color))?;
        }
        Ok(())
    }
    pub(crate) fn reset_fg_color(&mut self) -> std::io::Result<()> {
        if let Some(tty) = self.tty.as_mut() {
            write!(tty, "{}", color::Fg(color::Reset))?;
        }
        Ok(())
    }
}
