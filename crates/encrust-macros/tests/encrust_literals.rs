//! Tests for "encrusting" literals (numbers, strings and arrays and vecs of numbers or strings).

// Required because the macros expands to call functions from "encrust" crate, which cannot be
// imported into encrust_macros as this would introduce cyclic dependencies.
extern crate encrust_core as encrust;

use encrust_macros::encrust;

const TEST_STRING: &str = "The quick brown fox jumps over the lazy dog😊";

#[test]
fn encrust_ints() {
    let mut n = encrust!(1u8);
    let decrusted = n.decrust();
    assert_eq!(1u8, *decrusted);
    let mut n = encrust!(-1i8);
    let decrusted = n.decrust();
    assert_eq!(-1i8, *decrusted);
    let mut n = encrust!(1u16);
    let decrusted = n.decrust();
    assert_eq!(1u16, *decrusted);
    let mut n = encrust!(-1i16);
    let decrusted = n.decrust();
    assert_eq!(-1i16, *decrusted);
    let mut n = encrust!(1u32);
    let decrusted = n.decrust();
    assert_eq!(1u32, *decrusted);
    let mut n = encrust!(-1i32);
    let decrusted = n.decrust();
    assert_eq!(-1i32, *decrusted);
    let mut n = encrust!(1u64);
    let decrusted = n.decrust();
    assert_eq!(1u64, *decrusted);
    let mut n = encrust!(-1i64);
    let decrusted = n.decrust();
    assert_eq!(-1i64, *decrusted);
    let mut n = encrust!(1u128);
    let decrusted = n.decrust();
    assert_eq!(1u128, *decrusted);
    let mut n = encrust!(-1i128);
    let decrusted = n.decrust();
    assert_eq!(-1i128, *decrusted);
    let mut n = encrust!(1usize);
    let decrusted = n.decrust();
    assert_eq!(1usize, *decrusted);
    let mut n = encrust!(-1isize);
    let decrusted = n.decrust();
    assert_eq!(-1isize, *decrusted);
}

#[test]
fn encrust_string() {
    let mut s = encrust!("The quick brown fox jumps over the lazy dog😊");
    let decrusted = s.decrust();
    assert_eq!(TEST_STRING, &*decrusted);
}

#[test]
fn encrust_arrays() {
    const ORIG_ARRAY: [u8; 8] = [0, 1, 2, 3, 4, 5, 6, 7];
    
    let mut encrusted = encrust!([0u8, 1u8, 2u8, 3u8, 4u8, 5u8, 6u8, 7u8]);
    let decrusted = encrusted.decrust();
    assert_eq!(ORIG_ARRAY, &*decrusted);
}
