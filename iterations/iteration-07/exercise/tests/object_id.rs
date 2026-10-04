use rgit::{ObjectId, ParseObjectIdError};

#[test]
fn hash_of_blob_can_be_shown_shortened_and_parsed_back() {
    let id = rgit::hash_blob(b"hello\n");
    assert_eq!(id.to_string(), "ce013625030ba8dba906f756967f9e9ca394464a");
    assert_eq!(id.short(), "ce01362");
    assert_eq!(
        "CE013625030BA8DBA906F756967F9E9CA394464A".parse::<ObjectId>(),
        Ok(id)
    );
}

#[test]
fn short_hex_is_not_an_object_id() {
    assert_eq!(
        "ce01".parse::<ObjectId>(),
        Err(ParseObjectIdError::InvalidLength(4))
    );
}
