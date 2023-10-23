use std::io::BufReader;

use super::*;

enum TestFile {
    GetPods,
    PodYaml,
    Csv,
    GitStatus,
}

impl TestFile {
    fn to_str(self) -> &'static str {
        match self {
            TestFile::GetPods => include_str!("files/k-get-po.txt"),
            TestFile::PodYaml => include_str!("files/pod.yaml"),
            TestFile::Csv => include_str!("files/sample-vocabulary.csv"),
            TestFile::GitStatus => include_str!("files/git-status.txt"),
        }
    }
}

fn test_editor(test_file: TestFile) -> Editor {
    let buf = BufReader::new(stringreader::StringReader::new(test_file.to_str()));
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

    #[test]
    fn test_empty_token_rows() {
        test_key_seq(
            TestFile::GitStatus,
            vec![Key::Down; 9],
            "src/tests/files/git-status.txt",
        );
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

mod search {
    use super::*;
    #[test]
    fn test_search_kind() {
        test_key_seq(
            TestFile::PodYaml,
            vec![
                Key::Char('/'),
                Key::Char('k'),
                Key::Char('i'),
                Key::Char('n'),
                Key::Char('d'),
            ],
            "kind",
        );
    }

    #[test]
    fn test_search_king() {
        let keys = vec![
            Key::Char('/'),
            Key::Char('k'),
            Key::Char('i'),
            Key::Char('n'),
            Key::Char('g'),
            Key::Char('\n'),
        ];

        let mut editor = test_editor(TestFile::PodYaml);
        for key in keys {
            editor.process_keypress(key).unwrap();
        }
        assert_eq!(editor.should_quit, ShouldQuit::Ye(CopyStatus::Noop));
    }

    #[test]
    fn test_search_default() {
        let keys = vec![Key::Char('/'), Key::Char('\n')];

        let mut editor = test_editor(TestFile::PodYaml);
        for key in keys {
            editor.process_keypress(key).unwrap();
        }
        assert_eq!(editor.should_quit, ShouldQuit::Ye(CopyStatus::Noop));
    }

    #[test]
    fn test_search_and_normal() {
        test_key_seq(
            TestFile::PodYaml,
            vec![
                Key::Char('/'),
                Key::Char('c'),
                Key::Char('r'),
                Key::Char('e'),
                Key::Char('a'),
                Key::Esc,
                Key::Char('\n'),
            ],
            "creationTimestamp:",
        );
    }

    #[test]
    fn test_search_wraps_and_normal() {
        test_key_seq(
            TestFile::PodYaml,
            vec![
                Key::Char('/'),
                Key::Char('a'),
                Key::Char('n'),
                Key::Left,
                Key::Esc,
                Key::Char('\n'),
            ],
            "Guaranteed",
        );
    }
}

mod visual {
    use super::*;

    #[test]
    fn test_visual_mode() {
        test_key_seq(
            TestFile::PodYaml,
            vec![
                Key::Down,
                Key::Down,
                Key::Down,
                Key::Down,
                Key::Down,
                Key::Char('v'),
                Key::Char('v'),
                Key::Char('$'),
            ],
            "gatekeeper.sh/mutations: AssignMetadata//cluster-name-tag-datadog-agent:1, ModifySet//allow-scheduling-on-meta-node-pools:1",
        );
    }
}
