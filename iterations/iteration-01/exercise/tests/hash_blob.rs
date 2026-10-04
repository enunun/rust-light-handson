// 期待値は，同じ内容を`git hash-object --stdin`に渡して得た値である．

#[test]
fn hash_matches_git_for_text() {
    assert_eq!(
        rgit::hash_blob(b"hello\n"),
        "ce013625030ba8dba906f756967f9e9ca394464a"
    );
}

#[test]
fn hash_matches_git_for_bytes_that_are_not_text() {
    assert_eq!(
        rgit::hash_blob(&[0x00, 0xff, 0x10]),
        "d553b66b6a09553981f4c9b617e12de89b8fe30c"
    );
}
