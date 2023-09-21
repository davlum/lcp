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

enum TestFile {
    GetPods,
    PodYaml,
    Csv,
}

impl TestFile {
    fn to_str(self) -> &'static str {
        match self {
            TestFile::GetPods => include_str!("files/k-get-po.txt"),
            TestFile::PodYaml => include_str!("files/pod.yaml"),
            TestFile::Csv => include_str!("files/sample-vocabulary.csv"),
        }
    }
}

fn test_editor(test_file: TestFile) -> Editor {
    let buf = BufReader::new(StringReader::new(test_file.to_str()));
    let document = Document::new(buf).unwrap();
    let terminal = Terminal::new(Some((150, 150))).unwrap();
    Editor::new(document, None, terminal).unwrap()
}

fn test_key_seq(test_file: TestFile, keys: Vec<Key>, expected: &'static str) {
    let mut editor = test_editor(test_file);
    assert_eq!(editor.should_quit, ShouldQuit::No);
    for key in keys {
        editor.process_keypress(key).unwrap();
    }
    editor.process_keypress(Key::Char('\n')).unwrap();
    assert_eq!(
        editor.should_quit,
        ShouldQuit::Ye(CopyStatus::Success(expected.to_string()))
    );
}
mod normal {
    use super::*;

    #[test]
    fn test_exit() {
        let mut editor = test_editor(TestFile::GetPods);
        assert_eq!(editor.should_quit, ShouldQuit::No);
        editor.process_keypress(Key::Esc).unwrap();
        assert_eq!(editor.should_quit, ShouldQuit::Ye(CopyStatus::Noop));
    }

    #[test]
    fn test_default_pos_copy() {
        test_key_seq(
            TestFile::GetPods,
            vec![],
            "logdb-shared-ingest-756cfb4c58-68pgk",
        );
    }

    #[test]
    fn test_down_copy() {
        test_key_seq(
            TestFile::GetPods,
            vec![Key::Down],
            "logdb-shared-ingest-756cfb4c58-h2cmm",
        );
    }

    #[test]
    fn test_left_copy() {
        test_key_seq(TestFile::GetPods, vec![Key::Left], "45h");
    }

    #[test]
    fn test_left_right_copy() {
        test_key_seq(
            TestFile::GetPods,
            vec![Key::Left, Key::Right],
            "logdb-shared-ingest-756cfb4c58-68pgk",
        );
    }

    #[test]
    fn test_up_copy() {
        test_key_seq(
            TestFile::GetPods,
            vec![Key::Up],
            "staging-cron-userbehavior-lastlogin-28086480-cjr7z",
        );
    }

    #[test]
    fn test_up_down_copy() {
        test_key_seq(
            TestFile::GetPods,
            vec![Key::Up, Key::Down],
            "logdb-shared-ingest-756cfb4c58-68pgk",
        );
    }

    #[test]
    fn test_up_po_copy() {
        test_key_seq(TestFile::PodYaml, vec![Key::Up], "startTime:");
    }

    #[test]
    fn test_down_nine_po_copy() {
        test_key_seq(TestFile::PodYaml, vec![Key::Down; 9], "creationTimestamp:");
    }
}

mod tokenizer {
    use super::*;

    #[test]
    fn test_csv_default() {
        test_key_seq(
            TestFile::Csv,
            vec![Key::Char('t'), Key::Char(','), Key::Char('\n')],
            "schemas",
        );
    }

    #[test]
    fn test_csv_up() {
        test_key_seq(
            TestFile::Csv,
            vec![Key::Char('t'), Key::Char(','), Key::Char('\n'), Key::Up],
            "parameters",
        );
    }

    #[test]
    fn test_csv_down_right() {
        test_key_seq(
            TestFile::Csv,
            vec![
                Key::Char('t'),
                Key::Char(','),
                Key::Char('\n'),
                Key::Down,
                Key::Right,
            ],
            "\"random\"",
        );
    }
}
