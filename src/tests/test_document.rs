use super::*;
use std::io::BufReader;

#[test]
fn test_find_a() {
    let s = include_str!("files/pod.yaml");
    let buf = BufReader::new(stringreader::StringReader::new(s));
    let doc = Document::new(buf).unwrap();
    assert_eq!(
        doc.find(
            "a",
            &Position {
                x: 0,
                y: 0,
                longest_row: doc.longest_row()
            },
            SearchDirection::Forward
        ),
        Some(Position {
            x: 0,
            y: 0,
            longest_row: doc.longest_row()
        })
    );
}

#[test]
fn test_find_an() {
    let s = include_str!("files/pod.yaml");
    let buf = BufReader::new(stringreader::StringReader::new(s));
    let doc = Document::new(buf).unwrap();
    assert_eq!(
        doc.find(
            "an",
            &Position {
                x: 0,
                y: 0,
                longest_row: doc.longest_row()
            },
            SearchDirection::Forward
        ),
        Some(Position {
            x: 2,
            y: 3,
            longest_row: doc.longest_row()
        })
    );
}

#[test]
fn test_find_an_again() {
    let s = include_str!("files/pod.yaml");
    let buf = BufReader::new(stringreader::StringReader::new(s));
    let doc = Document::new(buf).unwrap();
    assert_eq!(
        doc.find(
            "an",
            &Position {
                x: 3,
                y: 3,
                longest_row: doc.longest_row()
            },
            SearchDirection::Forward
        ),
        Some(Position {
            x: 10,
            y: 13,
            longest_row: doc.longest_row
        })
    );
}
