use chacha20::{cipher::KeyIvInit, Key, XChaCha8, XNonce};
use encrust_core::Encrustable;
use encrust_macros::*;
use zeroize::Zeroize;

const TEST_STRING: &str = "The quick brown fox jumps over the lazy dog😊";

#[derive(Clone, Debug, Encrustable, PartialEq, Zeroize)]
struct Named {
    byte: u8,
    int: i32,
    array: [u16; 7],
    vec: Vec<i8>,
    string: String,
}

#[derive(Clone, Debug, Encrustable, PartialEq, Zeroize)]
struct Tuple(u8, i32, [u16; 7], Vec<i8>, String);

#[derive(Encrustable, Zeroize)]
struct _Unit;

#[derive(Clone, Debug, Encrustable, PartialEq, Zeroize)]
enum NamedOrTuple {
    Named {
        byte: u8,
        int: i32,
        array: [u16; 7],
        vec: Vec<i8>,
        string: String,
    },
    Tuple(u8, i32, [u16; 7], Vec<i8>, String),
    _Unit,
}

// Some bounds to check that bounds generation in the derive macro works
#[derive(Clone, Debug, Encrustable, PartialEq, Zeroize)]
struct Generic<T, U: PartialEq, P: Encrustable>(T, U, P);

fn gen_key_nonce() -> (Key, XNonce) {
    let key = Key::from([0x55; 32]);
    let nonce = XNonce::from([0xAA; 24]);

    (key, nonce)
}

#[test]
fn derive_named() {
    let named = Named {
        byte: 31,
        int: 1337,
        array: [6, 5, 4, 3, 2, 1, 0],
        vec: vec![13, 37],
        string: TEST_STRING.to_string(),
    };
    let original = named.clone();

    let (key, nonce) = gen_key_nonce();

    let mut encrusted = encrust_core::Encrusted::new(named, key, nonce);
    let decrusted = encrusted.decrust();
    assert!(decrusted.eq(&original));
}

#[test]
fn derive_named_ne() {
    let mut named = Named {
        byte: 31,
        int: 1337,
        array: [6, 5, 4, 3, 2, 1, 0],
        vec: vec![13, 37],
        string: TEST_STRING.to_string(),
    };
    let original = named.clone();

    let (key, nonce) = gen_key_nonce();

    let mut encruster = XChaCha8::new(&key, &nonce);
    unsafe {
        named.toggle_encrust(&mut encruster);
    }

    let mut encrusted = encrust_core::Encrusted::new(named, key, nonce);
    let decrusted = encrusted.decrust();

    assert!(decrusted.byte.ne(&original.byte));
    assert!(decrusted.int.ne(&original.int));
    assert!(decrusted.array.ne(&original.array));
    assert!(decrusted.vec.ne(&original.vec));
    assert!(decrusted.string.as_bytes().ne(original.string.as_bytes()));
}

#[test]
fn derive_tuple() {
    let named = Tuple(
        31,
        1337,
        [6, 5, 4, 3, 2, 1, 0],
        vec![13, 37],
        TEST_STRING.to_string(),
    );
    let original = named.clone();

    let (key, nonce) = gen_key_nonce();

    let mut encrusted = encrust_core::Encrusted::new(named, key, nonce);
    let decrusted = encrusted.decrust();
    assert!(decrusted.eq(&original));
}

#[test]
fn derive_tuple_ne() {
    let mut named = Tuple(
        31,
        1337,
        [6, 5, 4, 3, 2, 1, 0],
        vec![13, 37],
        TEST_STRING.to_string(),
    );
    let original = named.clone();

    let (key, nonce) = gen_key_nonce();

    let mut encruster = XChaCha8::new(&key, &nonce);
    unsafe {
        named.toggle_encrust(&mut encruster);
    }

    let mut encrusted = encrust_core::Encrusted::new(named, key, nonce);
    let decrusted = encrusted.decrust();

    assert!(decrusted.0.ne(&original.0));
    assert!(decrusted.1.ne(&original.1));
    assert!(decrusted.2.ne(&original.2));
    assert!(decrusted.3.ne(&original.3));
    assert!(decrusted.4.as_bytes().ne(original.4.as_bytes()));
}

#[test]
fn derive_enum_named() {
    let named = NamedOrTuple::Named {
        byte: 31,
        int: 1337,
        array: [6, 5, 4, 3, 2, 1, 0],
        vec: vec![13, 37],
        string: TEST_STRING.to_string(),
    };
    let original = named.clone();

    let (key, nonce) = gen_key_nonce();

    let mut encrusted = encrust_core::Encrusted::new(named, key, nonce);
    let decrusted = encrusted.decrust();
    assert!(decrusted.eq(&original));
}

#[test]
fn derive_enum_named_ne() {
    let mut named = NamedOrTuple::Named {
        byte: 31,
        int: 1337,
        array: [6, 5, 4, 3, 2, 1, 0],
        vec: vec![13, 37],
        string: TEST_STRING.to_string(),
    };
    let original = named.clone();

    let (key, nonce) = gen_key_nonce();

    let mut encruster = XChaCha8::new(&key, &nonce);
    unsafe {
        named.toggle_encrust(&mut encruster);
    }

    let mut encrusted = encrust_core::Encrusted::new(named, key, nonce);
    let decrusted = encrusted.decrust();

    match (&*decrusted, &original) {
        (
            NamedOrTuple::Named {
                byte,
                int,
                array,
                vec,
                string,
            },
            NamedOrTuple::Named {
                byte: orig_byte,
                int: orig_int,
                array: orig_array,
                vec: orig_vec,
                string: orig_string,
            },
        ) => {
            assert!(byte.ne(orig_byte));
            assert!(int.ne(orig_int));
            assert!(array.ne(orig_array));
            assert!(vec.ne(orig_vec));
            assert!(string.as_bytes().ne(orig_string.as_bytes()));
        }

        _ => panic!("Enum kinds should be Named but are not!?"),
    };
}

#[test]
fn derive_enum_tuple() {
    let tuple = NamedOrTuple::Tuple(
        31,
        1337,
        [6, 5, 4, 3, 2, 1, 0],
        vec![13, 37],
        TEST_STRING.to_string(),
    );
    let original = tuple.clone();

    let (key, nonce) = gen_key_nonce();

    let mut encrusted = encrust_core::Encrusted::new(tuple, key, nonce);
    let decrusted = encrusted.decrust();
    assert!(decrusted.eq(&original));
}

#[test]
fn derive_enum_tuple_ne() {
    let mut tuple = NamedOrTuple::Tuple(
        31,
        1337,
        [6, 5, 4, 3, 2, 1, 0],
        vec![13, 37],
        TEST_STRING.to_string(),
    );
    let original = tuple.clone();

    let (key, nonce) = gen_key_nonce();

    let mut encruster = XChaCha8::new(&key, &nonce);
    unsafe {
        tuple.toggle_encrust(&mut encruster);
    }

    let mut encrusted = encrust_core::Encrusted::new(tuple, key, nonce);
    let decrusted = encrusted.decrust();

    match (&*decrusted, &original) {
        (
            NamedOrTuple::Tuple(byte, int, array, vec, string),
            NamedOrTuple::Tuple(orig_byte, orig_int, orig_array, orig_vec, orig_string),
        ) => {
            assert!(byte.ne(orig_byte));
            assert!(int.ne(orig_int));
            assert!(array.ne(orig_array));
            assert!(vec.ne(orig_vec));
            assert!(string.as_bytes().ne(orig_string.as_bytes()));
        }

        _ => panic!("Enum kinds should be Tuple but are not!?"),
    }
}

#[test]
fn derive_with_generics() {
    let generic = Generic(13u32, [80u8; 4], TEST_STRING.to_string());
    let original = generic.clone();

    let (key, nonce) = gen_key_nonce();

    let mut encrusted = encrust_core::Encrusted::new(generic, key, nonce);
    let decrusted = encrusted.decrust();

    assert!(decrusted.eq(&original));
}

#[test]
fn derive_with_generics_ne() {
    let mut generic = Generic(13u32, [80u8; 4], TEST_STRING.to_string());
    let original = generic.clone();

    let (key, nonce) = gen_key_nonce();

    let mut encruster = XChaCha8::new(&key, &nonce);
    unsafe {
        generic.toggle_encrust(&mut encruster);
    }

    let mut encrusted = encrust_core::Encrusted::new(generic, key, nonce);
    let decrusted = encrusted.decrust();

    assert!(decrusted.0.ne(&original.0));
    assert!(decrusted.1.ne(&original.1));
    assert!(decrusted.2.as_bytes().ne(original.2.as_bytes()));
}
