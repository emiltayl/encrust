// unicode for good measure 🕶️

#[test]
fn encrust_file_string() {
    let orig_file = include_str!("encrust_files.rs");
    let mut encrust_file = encrust_macros::encrust_file_string!("tests/encrust_files.rs");
    let file = encrust_file.decrust();

    assert_eq!(orig_file, file.as_str());
}

#[test]
fn encrust_file_bytes() {
    let orig_file = include_bytes!("encrust_files.rs");
    let mut encrust_file = encrust_macros::encrust_file_bytes!("tests/encrust_files.rs");
    let file = encrust_file.decrust();

    assert_eq!(orig_file.as_slice(), file.as_slice());
}
