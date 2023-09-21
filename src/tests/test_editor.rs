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

fn test_editor_w_get_po() -> Editor {
    let buf = BufReader::new(StringReader::new(include_str!("k-get-po.txt")));
    let document = Document::new(buf).unwrap();
    let terminal = Terminal::new(Some((150, 150))).unwrap();
    Editor::new(document, None, terminal).unwrap()
}

fn test_editor_w_po() -> Editor {
    let buf = BufReader::new(StringReader::new(include_str!("pod.yaml")));
    let document = Document::new(buf).unwrap();
    let terminal = Terminal::new(Some((150, 150))).unwrap();
    Editor::new(document, None, terminal).unwrap()
}

fn test_get_po_key_seq(keys: Vec<Key>, expected: &'static str) {
    let mut editor = test_editor_w_get_po();
    assert!(!editor.should_quit);
    for key in keys {
        editor.process_keypress(key).unwrap();
    }
    editor.process_keypress(Key::Char('\n')).unwrap();
    editor.should_quit;
    assert_eq!(
        editor.copy_status,
        CopyStatus::Success(expected.to_string())
    );
}

fn test_po_key_seq(keys: Vec<Key>, expected: &'static str) {
    let mut editor = test_editor_w_po();
    assert!(!editor.should_quit);
    for key in keys {
        editor.process_keypress(key).unwrap();
    }
    editor.process_keypress(Key::Char('\n')).unwrap();
    editor.should_quit;
    assert_eq!(
        editor.copy_status,
        CopyStatus::Success(expected.to_string())
    );
}

#[test]
fn test_exit() {
    let mut editor = test_editor_w_get_po();
    assert!(!editor.should_quit);
    editor.process_keypress(Key::Esc).unwrap();
    assert!(editor.should_quit);
    assert_eq!(editor.copy_status, CopyStatus::Noop);
}

#[test]
fn test_default_pos_copy() {
    test_get_po_key_seq(vec![], "logdb-shared-ingest-756cfb4c58-68pgk");
}

#[test]
fn test_down_copy() {
    test_get_po_key_seq(vec![Key::Down], "logdb-shared-ingest-756cfb4c58-h2cmm");
}

#[test]
fn test_left_copy() {
    test_get_po_key_seq(vec![Key::Left], "45h");
}

#[test]
fn test_left_right_copy() {
    test_get_po_key_seq(
        vec![Key::Left, Key::Right],
        "logdb-shared-ingest-756cfb4c58-68pgk",
    );
}

#[test]
fn test_up_copy() {
    test_get_po_key_seq(
        vec![Key::Up],
        "staging-cron-userbehavior-lastlogin-28086480-cjr7z",
    );
}

#[test]
fn test_up_down_copy() {
    test_get_po_key_seq(
        vec![Key::Up, Key::Down],
        "logdb-shared-ingest-756cfb4c58-68pgk",
    );
}

#[test]
fn test_up_po_copy() {
    test_po_key_seq(vec![Key::Up], "startTime:");
}

#[test]
fn test_down_nine_po_copy() {
    test_po_key_seq(vec![Key::Down; 9], "creationTimestamp:");
}
