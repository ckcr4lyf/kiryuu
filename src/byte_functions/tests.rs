#[test]
fn is_legit(){
    assert_eq!("41", super::url_encoded_to_hex("A"));
    assert_eq!("41", super::url_encoded_to_hex("%41"));
    assert_eq!("4141", super::url_encoded_to_hex("A%41"));
    assert_eq!("4141", super::url_encoded_to_hex("%41A"));
    assert_eq!("4142", super::url_encoded_to_hex("%41B"));
    assert_eq!("4241", super::url_encoded_to_hex("B%41"));
    assert_eq!("4241", super::url_encoded_to_hex("BA"));
    assert_eq!("4142", super::url_encoded_to_hex("%41%42"));
}