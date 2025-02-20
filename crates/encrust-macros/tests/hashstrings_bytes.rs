//! Test of hashstrings macros.

// Required because the macros expands to call functions from "encrust" crate, which cannot be
// imported into encrust_macros as this would introduce cyclic dependencies.
extern crate encrust_core as encrust;

use encrust_macros::{hashbytes, hashstring, hashstring_ci};

const TEST_STRING: &str = "The quick brown fox jumps over the lazy dog😊";
const LOWERCASE_TEST_STRING: &str = "the quick brown fox jumps over the lazy dog😊";

#[test]
fn test_hashstrings() {
    let case_sensitive = hashstring!("The quick brown fox jumps over the lazy dog😊");
    let case_insensitive = hashstring_ci!("The quick brown fox jumps over the lazy dog😊");

    assert!(case_sensitive == TEST_STRING);
    assert!(case_insensitive == TEST_STRING);
    assert!(case_sensitive != LOWERCASE_TEST_STRING);
    assert!(case_insensitive == LOWERCASE_TEST_STRING);
}

#[test]
fn test_hashbytes() {
    let bytes = hashbytes!([0x0, 0b1, 2, 3, 4, 5]);

    assert!(bytes == &[0, 1, 2, 3, 4, 5]);
}
