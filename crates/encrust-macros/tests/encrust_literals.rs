//! Tests for "encrusting" literals (numbers, strings and arrays and vecs of numbers or strings).

// Required because the macros expands to call functions from "encrust" crate, which cannot be
// imported into encrust_macros as this would introduce cyclic dependencies.
extern crate encrust_core as encrust;

use std::ffi::CString;

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
fn encrust_empty_string_literals() {
    let mut s = encrust!("");
    assert_eq!("", &*s.decrust());

    let mut s = encrust!(b"");
    assert_eq!(b"", &*s.decrust());

    let mut s = encrust!(c"");
    assert_eq!(c"".to_bytes_with_nul(), &*s.decrust());
}

#[test]
fn encrust_bstring() {
    let mut s = encrust!(b"The quick brown fox jumps over the lazy dog");
    let decrusted = s.decrust();
    assert_eq!(b"The quick brown fox jumps over the lazy dog", &*decrusted);
}

#[test]
fn encrust_cstring() {
    let mut s = encrust!(c"The quick brown fox jumps over the lazy dog😊");
    let orig_cstring = CString::new(TEST_STRING).expect("CString::new failed.");
    let decrusted = s.decrust();

    assert_eq!(orig_cstring.as_bytes_with_nul(), &*decrusted);
}

#[test]
fn encrust_arrays() {
    const ORIG_ARRAY: [u8; 8] = [0, 1, 2, 3, 4, 5, 6, 7];
    const ORIG_ARRAYARRAY: [[u8; 8]; 2] = [[0, 1, 2, 3, 4, 5, 6, 7], [7, 6, 5, 4, 3, 2, 1, 0]];

    let mut encrusted = encrust!([0u8, 1u8, 2u8, 3u8, 4u8, 5u8, 6u8, 7u8]);
    let decrusted = encrusted.decrust();
    assert_eq!(ORIG_ARRAY, *decrusted);

    let mut encrusted = encrust!([
        [0u8, 1u8, 2u8, 3u8, 4u8, 5u8, 6u8, 7u8],
        [7u8, 6u8, 5u8, 4u8, 3u8, 2u8, 1u8, 0u8]
    ]);
    let decrusted = encrusted.decrust();
    assert_eq!(ORIG_ARRAYARRAY, *decrusted);
}

#[test]
fn encrust_integer_array_types() {
    let mut encrusted = encrust!([0u8, 1u8, 0xffu8]);
    assert_eq!([0u8, 1, 255], *encrusted.decrust());
    let mut encrusted = encrust!([0i8, -1i8, 0x7fi8]);
    assert_eq!([0i8, -1, 127], *encrusted.decrust());
    let mut encrusted = encrust!([0u16, 1u16, 0xffffu16]);
    assert_eq!([0u16, 1, u16::MAX], *encrusted.decrust());
    let mut encrusted = encrust!([0i16, -1i16, 0x7fffi16]);
    assert_eq!([0i16, -1, i16::MAX], *encrusted.decrust());
    let mut encrusted = encrust!([0u32, 1u32, 0xffff_ffffu32]);
    assert_eq!([0u32, 1, u32::MAX], *encrusted.decrust());
    let mut encrusted = encrust!([0i32, -1i32, 0x7fff_ffffi32]);
    assert_eq!([0i32, -1, i32::MAX], *encrusted.decrust());
    let mut encrusted = encrust!([0u64, 1u64, 0xffff_ffff_ffff_ffffu64]);
    assert_eq!([0u64, 1, u64::MAX], *encrusted.decrust());
    let mut encrusted = encrust!([0i64, -1i64, 0x7fff_ffff_ffff_ffffi64]);
    assert_eq!([0i64, -1, i64::MAX], *encrusted.decrust());
    let mut encrusted = encrust!([0u128, 1u128, 0xffff_ffff_ffff_ffff_ffff_ffff_ffff_ffffu128]);
    assert_eq!([0u128, 1, u128::MAX], *encrusted.decrust());
    let mut encrusted = encrust!([0i128, -1i128, 0x7fff_ffff_ffff_ffff_ffff_ffff_ffff_ffffi128]);
    assert_eq!([0i128, -1, i128::MAX], *encrusted.decrust());
}

#[test]
fn encrust_u8_array_lengths_around_chunk_boundaries() {
    let mut encrusted = encrust!([0u8]);
    assert_eq!([0u8], *encrusted.decrust());
    let mut encrusted = encrust!([0u8, 1u8]);
    assert_eq!([0u8, 1], *encrusted.decrust());
    let mut encrusted = encrust!([0u8, 1u8, 2u8, 3u8, 4u8, 5u8, 6u8]);
    assert_eq!([0u8, 1, 2, 3, 4, 5, 6], *encrusted.decrust());
    let mut encrusted = encrust!([0u8, 1u8, 2u8, 3u8, 4u8, 5u8, 6u8, 7u8]);
    assert_eq!([0u8, 1, 2, 3, 4, 5, 6, 7], *encrusted.decrust());
    let mut encrusted = encrust!([0u8, 1u8, 2u8, 3u8, 4u8, 5u8, 6u8, 7u8, 8u8]);
    assert_eq!([0u8, 1, 2, 3, 4, 5, 6, 7, 8], *encrusted.decrust());
    let mut encrusted = encrust!([
        0u8, 1u8, 2u8, 3u8, 4u8, 5u8, 6u8, 7u8, 8u8, 9u8, 10u8, 11u8, 12u8, 13u8, 14u8
    ]);
    assert_eq!(
        [0u8, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14],
        *encrusted.decrust()
    );
    let mut encrusted = encrust!([
        0u8, 1u8, 2u8, 3u8, 4u8, 5u8, 6u8, 7u8, 8u8, 9u8, 10u8, 11u8, 12u8, 13u8, 14u8, 15u8
    ]);
    assert_eq!(
        [0u8, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
        *encrusted.decrust()
    );
    let mut encrusted = encrust!([
        0u8, 1u8, 2u8, 3u8, 4u8, 5u8, 6u8, 7u8, 8u8, 9u8, 10u8, 11u8, 12u8, 13u8, 14u8, 15u8, 16u8
    ]);
    assert_eq!(
        [0u8, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
        *encrusted.decrust()
    );
}
