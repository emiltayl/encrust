#![deny(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(all(not(test), not(feature = "std")), no_std)]

//! Crate implementing core functionality for `encrust`. See the main crate for
//! documentation.

#[cfg(not(feature = "std"))]
extern crate core;

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};

#[cfg(not(feature = "std"))]
use core::ops::{Deref, DerefMut};

#[cfg(feature = "std")]
use std::ops::{Deref, DerefMut};

use chacha20::{
    cipher::{KeyIvInit, StreamCipher},
    Key, XChaCha8, XNonce,
};

use zeroize::Zeroize;

/// Container struct for encrust, accepting [`Encrustable`] + `Zeroize` types
/// for encryption and decryption when needed.
///
/// See [encrust](./index.html) for example usage.
pub struct Encrusted<T>
where
    T: Encrustable + Zeroize,
{
    data: T,
    key: Key,
    nonce: XNonce,
}

impl<T> Encrusted<T>
where
    T: Encrustable + Zeroize,
{
    /// Accepts [`Encrustable`] + `Zeroize` data and encrypts it using the
    /// provided `Key` and `XNonce`.
    pub fn new(mut data: T, key: Key, nonce: XNonce) -> Self {
        let mut encruster = XChaCha8::new(&key, &nonce);

        // SAFETY:
        // We take ownership of data and only expose it after calling
        // toggle_encrust another time, ensuring that the encrypted data is
        // not used.
        unsafe {
            data.toggle_encrust(&mut encruster);
        }

        Self { data, key, nonce }
    }

    /// Creates an "encrusted" object from pre-scrambeled data. This is used by
    /// macros to include pre-scrambled objects in the source and should not be
    /// called manually.
    ///
    /// # Safety
    /// Using this may cause data to be scrambled in unpredictable ways that
    /// could lead to safety issues. This should not be used manually, but
    /// solely through the provided macros.
    #[doc(hidden)]
    #[cfg_attr(docsrs, doc(cfg(feature = "macros")))]
    #[cfg(feature = "macros")]
    pub const unsafe fn from_encrusted_data(data: T, key: Key, nonce: XNonce) -> Self {
        Self { data, key, nonce }
    }

    /// Accepts [`Encrustable`] + `Zeroize` data and encrypts it using a
    /// randomly generated key and nonce.
    #[cfg_attr(docsrs, doc(cfg(feature = "rand")))]
    #[cfg(feature = "rand")]
    pub fn new_with_random<R>(data: T, mut rng: R) -> Self
    where
        R: rand::Rng,
    {
        let key: Key = rng.gen::<[u8; 32]>().into();
        let nonce: XNonce = rng.gen::<[u8; 24]>().into();

        Self::new(data, key, nonce)
    }

    /// Changes the key and nonce used to encrypt the underlying data.
    #[cfg_attr(docsrs, doc(cfg(feature = "rand")))]
    #[cfg(feature = "rand")]
    pub fn rekey<R>(&mut self, mut rng: R)
    where
        R: rand::Rng,
    {
        {
            let mut decruster = XChaCha8::new(&self.key, &self.nonce);

            // SAFETY:
            // In order to encrypt with a new key and nonce, the data needs to
            // be decrypted first. To be safe, this function needs to call
            // toggle_encrypt another time before returning.
            unsafe {
                self.data.toggle_encrust(&mut decruster);
            }
        }

        self.key = rng.gen::<[u8; 32]>().into();
        self.nonce = rng.gen::<[u8; 24]>().into();

        let mut encruster = XChaCha8::new(&self.key, &self.nonce);

        // SAFETY:
        // Encrypt the data again with a new key and nonce, this needs to happen
        // for Encrusted to work properly, otherwise we risk exposing encrypted
        // data when decrypted is expected.
        unsafe {
            self.data.toggle_encrust(&mut encruster);
        }
    }

    /// Decrypts the data contained in [`Encrusted`] and returns a [`Decrusted`]
    /// object that can be used to access and modify the actual data.
    pub fn decrust(&mut self) -> Decrusted<T> {
        Decrusted::new(self)
    }
}

impl<T> Drop for Encrusted<T>
where
    T: Encrustable + Zeroize,
{
    /// [`Encrusted`]'s drop implementation calls zeroize on the underlying data
    /// including the key and nonce to prevent secrets from staying in memory
    /// when they are no loger needed.
    fn drop(&mut self) {
        self.data.zeroize();
        self.key.zeroize();
        self.data.zeroize();
    }
}

/// Type used to access encrusted data.
///
/// Use [`Encrusted::decrust`] to create `Decrusted` objects. See
/// [encrust](./index.html) for example usage.
///
/// When the Decrusted object is dropped, the underlying data is re-encrypted.
pub struct Decrusted<'decrusted, T>
where
    T: Encrustable + Zeroize,
{
    encrusted_data: &'decrusted mut Encrusted<T>,
}

impl<'decrusted, T> Decrusted<'decrusted, T>
where
    T: Encrustable + Zeroize,
{
    fn new(encrusted_data: &'decrusted mut Encrusted<T>) -> Self {
        let mut decruster = XChaCha8::new(&encrusted_data.key, &encrusted_data.nonce);

        // SAFETY:
        // This needs to happen to decrypt the data for use as it is encrypted.
        unsafe {
            encrusted_data.data.toggle_encrust(&mut decruster);
        }

        Self { encrusted_data }
    }
}

impl<'decrusted, T> Drop for Decrusted<'decrusted, T>
where
    T: Encrustable + Zeroize,
{
    fn drop(&mut self) {
        let mut encruster = XChaCha8::new(&self.encrusted_data.key, &self.encrusted_data.nonce);

        // SAFETY:
        // This needs to happen to encrypt the data when this object is dropped
        // to ensure that data does not linger in memory unencrypted when not
        // needed.
        unsafe {
            self.encrusted_data.data.toggle_encrust(&mut encruster);
        }
    }
}

impl<'decrusted, T> Deref for Decrusted<'decrusted, T>
where
    T: Encrustable + Zeroize,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.encrusted_data.data
    }
}

impl<'decrusted, T> DerefMut for Decrusted<'decrusted, T>
where
    T: Encrustable + Zeroize,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.encrusted_data.data
    }
}

/// Trait required to use data types with encrust. If it is avoidable, do not
/// implement this manually, but use the derive macro to generate the
/// implementation.
pub trait Encrustable {
    /// Called when encrypting and decrypting data. Using this function manually
    /// may lead to safety issues and should not be called explicitly.
    ///
    /// # Safety
    /// `toggle_encrust` directly modifies the underlying data in arbitrary
    /// ways, possibly making it unsafe to use. This function should only ever
    /// be called by encrust to scramble objects or unscramble them for reading.
    unsafe fn toggle_encrust(&mut self, encruster: &mut XChaCha8);
}

macro_rules! encrustable_number {
    ( $( $t:ty ),* ) => {
        $(
            impl Encrustable for $t {
                unsafe fn toggle_encrust(&mut self, encruster: &mut XChaCha8) {
                    let mut bytes = self.to_ne_bytes();

                    encruster.apply_keystream(&mut bytes);

                    *self = Self::from_ne_bytes(bytes);
                }
            }
        )*
    };
}

encrustable_number!(u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize);

impl Encrustable for String {
    unsafe fn toggle_encrust(&mut self, encruster: &mut XChaCha8) {
        encruster.apply_keystream(self.as_mut_vec());
    }
}

impl<T, const N: usize> Encrustable for [T; N]
where
    T: Encrustable,
{
    unsafe fn toggle_encrust(&mut self, encruster: &mut XChaCha8) {
        for element in self {
            element.toggle_encrust(encruster);
        }
    }
}

impl<T> Encrustable for Vec<T>
where
    T: Encrustable,
{
    unsafe fn toggle_encrust(&mut self, encruster: &mut XChaCha8) {
        for element in self {
            element.toggle_encrust(encruster);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_STRING: &str = "The quick brown fox jumps over the lazy dog😊";

    fn get_key() -> Key {
        chacha20::Key::from([0x55; 32])
    }

    fn get_nonce() -> XNonce {
        chacha20::XNonce::from([0xAA; 24])
    }

    macro_rules! test_ints {
        ( $( $t:ty ),* ) => {
            $(
                {
                    let mut encrusted = Encrusted::<$t>::new(0, get_key(), get_nonce());
                    assert_ne!(encrusted.data, 0);

                    {
                        let decrusted = encrusted.decrust();
                        assert_eq!(*decrusted, 0);
                    }

                    assert_ne!(encrusted.data, 0);
                }

                {
                    let key = get_key();
                    let nonce = get_nonce();
                    let mut encruster = XChaCha8::new(&key, &nonce);
                    let mut encrusted_data: $t = 0;
                    let mut encrusted = unsafe {
                        encrusted_data.toggle_encrust(&mut encruster);
                        Encrusted::<$t>::from_encrusted_data(encrusted_data, key.into(), nonce.into())
                    };

                    assert_ne!(encrusted.data, 0);

                    {
                        let decrusted = encrusted.decrust();
                        assert_eq!(*decrusted, 0);
                    }

                    assert_ne!(encrusted.data, 0);
                }
            )*
        };
    }

    #[test]
    fn test_ints() {
        test_ints!(u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize);
    }

    #[test]
    fn test_strings() {
        let mut encrusted = Encrusted::new(TEST_STRING.to_string(), get_key(), get_nonce());
        assert_ne!(encrusted.data.as_bytes(), TEST_STRING.as_bytes());

        {
            let decrusted = encrusted.decrust();
            assert_eq!(*decrusted, TEST_STRING);
        }

        assert_ne!(encrusted.data.as_bytes(), TEST_STRING.as_bytes());
    }

    #[test]
    fn test_strings_from_encrusted() {
        let key = get_key();
        let nonce = get_nonce();
        let mut encruster = XChaCha8::new(&key, &nonce);

        let mut encrusted_string = TEST_STRING.to_string();

        let mut encrusted = unsafe {
            encrusted_string.toggle_encrust(&mut encruster);
            Encrusted::from_encrusted_data(encrusted_string, key.into(), nonce.into())
        };

        assert_ne!(encrusted.data.as_bytes(), TEST_STRING.as_bytes());

        {
            let decrusted = encrusted.decrust();
            assert_eq!(*decrusted, TEST_STRING);
        }

        assert_ne!(encrusted.data.as_bytes(), TEST_STRING.as_bytes());
    }

    #[test]
    fn test_arrays() {
        let orig_array: [u8; 45] = [
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44,
        ];

        let mut encrusted = Encrusted::new(orig_array.clone(), get_key(), get_nonce());
        assert_ne!(encrusted.data, orig_array);

        {
            let decrusted = encrusted.decrust();
            assert_eq!(*decrusted, orig_array);
        }

        assert_ne!(encrusted.data, orig_array);
    }

    #[test]
    fn test_arrays_from_encrusted() {
        let key = get_key();
        let nonce = get_nonce();
        let mut encruster = XChaCha8::new(&key, &nonce);
        let orig_array: [u8; 45] = [
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44,
        ];

        let mut encrusted_array = orig_array.clone();
        let mut encrusted = unsafe {
            encrusted_array.toggle_encrust(&mut encruster);
            Encrusted::from_encrusted_data(encrusted_array, get_key(), get_nonce())
        };

        assert_ne!(encrusted.data, orig_array);

        {
            let decrusted = encrusted.decrust();
            assert_eq!(*decrusted, orig_array);
        }

        assert_ne!(encrusted.data, orig_array);
    }

    #[test]
    fn test_vecs() {
        let orig_vec = TEST_STRING.as_bytes().to_vec();

        let mut encrusted = Encrusted::new(orig_vec.clone(), get_key(), get_nonce());
        assert_ne!(encrusted.data, orig_vec);

        {
            let decrusted = encrusted.decrust();
            assert_eq!(*decrusted, orig_vec);
        }

        assert_ne!(encrusted.data, orig_vec);
    }

    #[test]
    fn test_vecs_from_encrusted() {
        let key = get_key();
        let nonce = get_nonce();
        let mut encruster = XChaCha8::new(&key, &nonce);
        let orig_vec = TEST_STRING.as_bytes().to_vec();

        let mut encrusted_vec = orig_vec.clone();

        let mut encrusted = unsafe {
            encrusted_vec.toggle_encrust(&mut encruster);
            Encrusted::from_encrusted_data(encrusted_vec, get_key(), get_nonce())
        };

        assert_ne!(encrusted.data, orig_vec);

        {
            let decrusted = encrusted.decrust();
            assert_eq!(*decrusted, orig_vec);
        }

        assert_ne!(encrusted.data, orig_vec);
    }

    #[test]
    #[cfg(feature = "rand")]
    fn test_random_initializer() {
        let num = 828627825u64;
        let mut encrusted = Encrusted::new_with_random(num, rand::thread_rng());
        assert_ne!(encrusted.data, num);

        {
            let decrusted = encrusted.decrust();
            assert_eq!(*decrusted, num);
        }

        assert_ne!(encrusted.data, num);
    }

    #[test]
    #[cfg(feature = "rand")]
    fn test_rekey() {
        let num = 828627825u64;
        let mut encrusted = Encrusted::new(num, get_key(), get_nonce());
        let orig_key = encrusted.key.clone();
        let orig_nonce = encrusted.nonce.clone();

        encrusted.rekey(rand::thread_rng());

        // May fail, but both the key and nonce are so large that you should not see a collision in either if they are selected randomly
        assert_ne!(encrusted.key, orig_key);
        assert_ne!(encrusted.nonce, orig_nonce);

        {
            let decrusted = encrusted.decrust();
            assert_eq!(*decrusted, num);
        }
    }
}
