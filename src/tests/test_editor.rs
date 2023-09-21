use super::*;
use std::io::{BufReader, Read};
use std::slice::Iter;

pub struct StringReader<'a> {
    iter: Iter<'a, u8>,
}

impl<'a> StringReader<'a> {
    /// Wrap a string in a `StringReader`, which implements `std::io::Read`.
    pub fn new(data: &'a str) -> Self {
        Self {
            iter: data.as_bytes().iter(),
        }
    }
}

impl<'a> Read for StringReader<'a> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        for i in 0..buf.len() {
            if let Some(x) = self.iter.next() {
                buf[i] = *x;
            } else {
                return Ok(i);
            }
        }
        Ok(buf.len())
    }
}

#[test]
fn test_exit() {
    let buf = BufReader::new(StringReader::new(include_str!("k-get-po.txt")));
    let document = Document::new(buf).unwrap();
    let terminal = Terminal::new(Some((150, 150))).unwrap();
    let mut editor = Editor::new(document, None, terminal).unwrap();
    let key = Key::Esc;
    assert!(!editor.should_quit);
    editor.process_keypress(key).unwrap();
    assert!(editor.should_quit);
    assert_eq!(editor.copy_status, CopyStatus::Noop);
}

#[test]
fn test_default_pos_copy() {
    let buf = BufReader::new(StringReader::new(include_str!("k-get-po.txt")));
    let document = Document::new(buf).unwrap();
    let terminal = Terminal::new(Some((150, 150))).unwrap();
    let mut editor = Editor::new(document, None, terminal).unwrap();
    let key = Key::Char('\n');
    assert!(!editor.should_quit);
    editor.process_keypress(key).unwrap();
    assert!(editor.should_quit);
    assert_eq!(
        editor.copy_status,
        CopyStatus::Success("logdb-shared-ingest-756cfb4c58-68pgk".to_string())
    );
}
