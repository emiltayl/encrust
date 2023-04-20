use encrust_macros::{encrust, encrust_vec};

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
    assert_eq!(TEST_STRING, decrusted.as_str());
}

#[test]
fn encrust_arrays() {
    const ORIG_ARRAY: [[[u8; 3]; 3]; 3] = [
        [[1u8, 2u8, 3u8], [1u8, 2u8, 3u8], [1u8, 2u8, 3u8]],
        [[1u8, 2u8, 3u8], [1u8, 2u8, 3u8], [1u8, 2u8, 3u8]],
        [[1u8, 2u8, 3u8], [1u8, 2u8, 3u8], [1u8, 2u8, 3u8]],
    ];
    let mut aa = encrust!([
        [[1u8, 2u8, 3u8], [1u8, 2u8, 3u8], [1u8, 2u8, 3u8]],
        [[1u8, 2u8, 3u8], [1u8, 2u8, 3u8], [1u8, 2u8, 3u8]],
        [[1u8, 2u8, 3u8], [1u8, 2u8, 3u8], [1u8, 2u8, 3u8]]
    ]);
    let decrusted = aa.decrust();
    assert_eq!(ORIG_ARRAY, *decrusted);

    let mut sa = encrust!([
        "The quick brown fox jumps over the lazy dog😊",
        "The quick brown fox jumps over the lazy dog😊",
        "The quick brown fox jumps over the lazy dog😊"
    ]);
    let decrusted = sa.decrust();
    assert_eq!(
        [
            TEST_STRING.to_string(),
            TEST_STRING.to_string(),
            TEST_STRING.to_string()
        ],
        *decrusted
    );
}

#[test]
fn encrust_vec() {
    const ORIG_ARRAY: [u8; 27] = [
        1u8, 2u8, 3u8, 1u8, 2u8, 3u8, 1u8, 2u8, 3u8, 1u8, 2u8, 3u8, 1u8, 2u8, 3u8, 1u8, 2u8, 3u8,
        1u8, 2u8, 3u8, 1u8, 2u8, 3u8, 1u8, 2u8, 3u8,
    ];
    const ORIG_ARRAY2: [[[u8; 3]; 3]; 3] = [
        [[1u8, 2u8, 3u8], [1u8, 2u8, 3u8], [1u8, 2u8, 3u8]],
        [[1u8, 2u8, 3u8], [1u8, 2u8, 3u8], [1u8, 2u8, 3u8]],
        [[1u8, 2u8, 3u8], [1u8, 2u8, 3u8], [1u8, 2u8, 3u8]],
    ];
    let orig_array3: [String; 4] = [
        TEST_STRING.to_string(),
        TEST_STRING.to_string(),
        TEST_STRING.to_string(),
        TEST_STRING.to_string(),
    ];

    let mut vec = encrust_vec![
        1u8, 2u8, 3u8, 1u8, 2u8, 3u8, 1u8, 2u8, 3u8, 1u8, 2u8, 3u8, 1u8, 2u8, 3u8, 1u8, 2u8, 3u8,
        1u8, 2u8, 3u8, 1u8, 2u8, 3u8, 1u8, 2u8, 3u8
    ];
    let decrusted = vec.decrust();
    assert_eq!(ORIG_ARRAY.to_vec(), *decrusted);

    let mut vec = encrust_vec![
        [[1u8, 2u8, 3u8], [1u8, 2u8, 3u8], [1u8, 2u8, 3u8]],
        [[1u8, 2u8, 3u8], [1u8, 2u8, 3u8], [1u8, 2u8, 3u8]],
        [[1u8, 2u8, 3u8], [1u8, 2u8, 3u8], [1u8, 2u8, 3u8]],
    ];
    let decrusted = vec.decrust();
    assert_eq!(ORIG_ARRAY2.to_vec(), *decrusted);

    let mut vec = encrust_vec![
        "The quick brown fox jumps over the lazy dog😊",
        "The quick brown fox jumps over the lazy dog😊",
        "The quick brown fox jumps over the lazy dog😊",
        "The quick brown fox jumps over the lazy dog😊",
    ];
    let decrusted = vec.decrust();
    assert_eq!(orig_array3.to_vec(), *decrusted);
}
