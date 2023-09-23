use super::*;
use std::io::BufReader;

#[test]
fn test_find_a() {
    let s = include_str!("files/pod.yaml");
    let buf = BufReader::new(stringreader::StringReader::new(s));
    let document = Document::new(buf).unwrap();
    assert_eq!(
        document.find("a", &Position { x: 0, y: 0 }, SearchDirection::Forward),
        Some(Position { x: 0, y: 0 })
    );
}

#[test]
fn test_find_an() {
    let s = include_str!("files/pod.yaml");
    let buf = BufReader::new(stringreader::StringReader::new(s));
    let document = Document::new(buf).unwrap();
    assert_eq!(
        document.find("an", &Position { x: 0, y: 0 }, SearchDirection::Forward),
        Some(Position { x: 2, y: 3 })
    );
}

#[test]
fn test_find_an_again() {
    let s = include_str!("files/pod.yaml");
    let buf = BufReader::new(stringreader::StringReader::new(s));
    let document = Document::new(buf).unwrap();
    assert_eq!(
        document.find("an", &Position { x: 3, y: 3 }, SearchDirection::Forward),
        Some(Position { x: 10, y: 13 })
    );
}
