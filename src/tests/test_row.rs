use super::*;

#[test]
fn test_find() {
    let row = Row::new("1testtest", &Separator::Whitespace);
    assert_eq!(row.find("t", 0, SearchDirection::Forward), Some(1));
    assert_eq!(row.find("t", 2, SearchDirection::Forward), Some(4));
    assert_eq!(row.find("t", 5, SearchDirection::Forward), Some(5));
}

#[test]
fn test_row() {
    let expected = vec![
        Token { start: 0, len: 36 },
        Token { start: 66, len: 3 },
        Token { start: 74, len: 7 },
        Token { start: 86, len: 1 },
        Token { start: 102, len: 3 },
    ];
    let s = "logdb-shared-ingest-756cfb4c58-68pgk                              1/1     Running     0               45h";
    let row = Row::new(s, &Separator::Whitespace);
    assert_eq!(row.tokens, expected);
}

#[test]
fn test_row_starts_whitespace() {
    let expected = vec![Token { start: 4, len: 47 }, Token { start: 52, len: 5 }];
    let s = "    cluster-autoscaler.kubernetes.io/safe-to-evict: false";
    let row = Row::new(s, &Separator::Whitespace);
    assert_eq!(row.tokens, expected);
}
