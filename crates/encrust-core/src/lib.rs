#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(all(not(test), not(feature = "std")), no_std)]

//! Crate implementing core functionality for `encrust`. See the main crate for documentation.

#[cfg(feature = "hashstrings")]
mod hashstrings;
#[cfg(feature = "hashstrings")]
pub use hashstrings::*;

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

use rand::{RngCore, SeedableRng, rngs::SmallRng};
use zeroize::Zeroize;

/// Container struct for encrust, accepting [`Encrustable`] + `Zeroize` types for obfuscation and
/// deobfuscation when needed.
///
/// Care should be taken if `T` has a non-trivial `Drop` implementation, as `T` is not dropped until
/// `zeroize` has been called on it.
pub struct Encrusted<T>
where
    T: Encrustable + Zeroize,
{
    data: T,
    seed: u64,
}

impl<T> Encrusted<T>
where
    T: Encrustable + Zeroize,
{
    /// Accepts [`Encrustable`] + `Zeroize` data and obfuscates it using the provided seed.
    pub fn new(mut data: T, seed: u64) -> Self {
        let mut encrust_rng = SmallRng::seed_from_u64(seed);

        // SAFETY:
        // `Encrusted` takes ownership of the data and only exposes it after calling toggle_encrust
        // again, ensuring that the underlying data is not accessed in a potential invalid state.
        unsafe {
            data.toggle_encrust(&mut encrust_rng);
        }

        Self { data, seed }
    }

    /// Creates an `Encrusted` object from pre-scrambeled data. This is used by macros to include
    /// pre-scrambled objects in the source and should not be called manually.
    ///
    /// # Safety
    /// Using this may cause data to be scrambled in unpredictable ways that could lead to safety
    /// issues. This should not be used manually, but only through the provided macros.
    #[doc(hidden)]
    #[cfg(feature = "macros")]
    pub const unsafe fn from_encrusted_data(data: T, seed: u64) -> Self {
        Self { data, seed }
    }

    /// Changes the seed used to obfuscate the underlying data.
    pub fn reseed(&mut self, new_seed: u64) {
        {
            let mut decruster = SmallRng::seed_from_u64(self.seed);

            // SAFETY:
            // In order to obfuscate with a new seed, the data needs to be deobfuscated first.
            unsafe {
                self.data.toggle_encrust(&mut decruster);
            }
        }

        self.seed = new_seed;

        let mut encrust_rng = SmallRng::seed_from_u64(self.seed);

        // SAFETY:
        // Obsucate the data again with a new seed.
        unsafe {
            self.data.toggle_encrust(&mut encrust_rng);
        }
    }

    /// Deobfuscates the data contained in [`Encrusted`] and returns a [`Decrusted`] object that can
    /// be used to access and modify the actual data.
    pub fn decrust(&mut self) -> Decrusted<'_, T> {
        Decrusted::new(self)
    }
}

impl<T> Drop for Encrusted<T>
where
    T: Encrustable + Zeroize,
{
    /// [`Encrusted`]'s drop implementation calls zeroize on the underlying data including the seed
    /// to prevent secrets from staying in memory when they are no longer needed.
    ///
    /// Note that the data is zeroized prior to being dropped, which may cause problems for the drop
    /// implementation of the underlying data.
    fn drop(&mut self) {
        self.data.zeroize();
        self.seed.zeroize();
    }
}

/// Type used to access encrusted data. Use [`Encrusted::decrust`] to create `Decrusted` data.
///
/// When the `Decrusted` object is dropped, the underlying data is re-obfuscated.
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
        let mut decruster = SmallRng::seed_from_u64(encrusted_data.seed);

        // SAFETY:
        // This needs to happen to deobfuscate the data for use. Without this, invalid data can
        // cause problems, such as strings with data that is not valid UTF-8.
        unsafe {
            encrusted_data.data.toggle_encrust(&mut decruster);
        }

        Self { encrusted_data }
    }
}

impl<T> Drop for Decrusted<'_, T>
where
    T: Encrustable + Zeroize,
{
    fn drop(&mut self) {
        let mut encrust_rng = SmallRng::seed_from_u64(self.encrusted_data.seed);

        // SAFETY:
        // This needs to happen to obfuscate the data when this object is dropped to ensure that
        // data does not linger in memory unobfuscated when not needed. Data will not be accessible
        // without deobfuscating the data, so this should not cause any issues.
        unsafe {
            self.encrusted_data.data.toggle_encrust(&mut encrust_rng);
        }
    }
}

impl<T> Deref for Decrusted<'_, T>
where
    T: Encrustable + Zeroize,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.encrusted_data.data
    }
}

impl<T> DerefMut for Decrusted<'_, T>
where
    T: Encrustable + Zeroize,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.encrusted_data.data
    }
}

/// Trait required to use data types with encrust. If it is avoidable, do not implement this
/// manually, but use the derive macro to generate the implementation.
pub trait Encrustable {
    /// Called when obfuscating and deobfuscating data. Calling this function manually may lead to
    /// safety issues and should not be done.
    ///
    /// # Safety
    /// `toggle_encrust` directly modifies the underlying data in arbitrary ways, possibly making it
    /// unsafe to use. This function should only ever be called by encrust to obfuscate objects or
    /// deobfuscate them for reading.
    unsafe fn toggle_encrust(&mut self, encrust_rng: &mut impl RngCore);
}

macro_rules! encrustable_number {
    ( $( $t:ty ),* ) => {
        $(
            impl Encrustable for $t {
                unsafe fn toggle_encrust(&mut self, encrust_rng: &mut impl ::rand::RngCore) {
                    let mut bytes = self.to_le_bytes();

                    // Using 8 bytes as most numbers that will be used with encrust are (most
                    // likely) 64-bit or smaller.
                    let mut key: [u8; 8] = [0; 8];
                    for chunk in bytes.chunks_mut(8) {
                        encrust_rng.fill_bytes(&mut key);
                        for (byte, byte_key) in chunk.iter_mut().zip(key.iter()) {
                            *byte ^= byte_key;
                        }
                    }

                    *self = Self::from_le_bytes(bytes);
                }
            }
        )*
    };
}

encrustable_number!(
    u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize
);

impl Encrustable for String {
    unsafe fn toggle_encrust(&mut self, encrust_rng: &mut impl RngCore) {
        // Safety: This modifies the underlying bytes directly, which is unsafe. However, the
        // changes are reverted before granting access to the underlying memory again.
        let bytes = unsafe { self.as_mut_vec() };

        // Encrusting 16 bytes at a time as a micro-benchmark showed that it was most efficient on
        // the tested x86-64 systems.
        let mut key: [u8; 16] = [0; 16];
        for chunk in bytes.chunks_mut(16) {
            encrust_rng.fill_bytes(&mut key);
            for (byte, byte_key) in chunk.iter_mut().zip(key.iter()) {
                *byte ^= byte_key;
            }
        }
    }
}

impl<T, const N: usize> Encrustable for [T; N]
where
    T: Encrustable,
{
    unsafe fn toggle_encrust(&mut self, encrust_rng: &mut impl RngCore) {
        for element in self {
            // Safety: This modifies the underlying bytes directly, which is unsafe. However, the
            // changes are reverted before granting access to the underlying memory again.
            unsafe {
                element.toggle_encrust(encrust_rng);
            }
        }
    }
}

impl<T> Encrustable for Vec<T>
where
    T: Encrustable,
{
    unsafe fn toggle_encrust(&mut self, encrust_rng: &mut impl RngCore) {
        for element in self {
            // Safety: This modifies the underlying bytes directly, which is unsafe. However, the
            // changes are reverted before granting access to the underlying memory again.
            unsafe {
                element.toggle_encrust(encrust_rng);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_STRING: &str = "The quick brown fox jumps over the lazy dog😊";

    fn get_seed() -> u64 {
        0x2357_bd11_1317_1d1f
    }

    macro_rules! test_ints {
        ( $( $t:ty ),* ) => {
            $(
                {
                    let mut encrusted = Encrusted::<$t>::new(0, get_seed());
                    assert_ne!(encrusted.data, 0);

                    {
                        let decrusted = encrusted.decrust();
                        assert_eq!(*decrusted, 0);
                    }

                    assert_ne!(encrusted.data, 0);
                }

                {
                    let seed = get_seed();
                    let mut encrust_rng = SmallRng::seed_from_u64(seed);
                    let mut encrusted_data: $t = 0;

                    // Safety: Testing from_encrusted_data requires pre-encrusted data, which is
                    // an unsafe operation. The data will not be available without calling
                    // `toggle_encrust` again.
                    let mut encrusted = unsafe {
                        encrusted_data.toggle_encrust(&mut encrust_rng);
                        Encrusted::<$t>::from_encrusted_data(encrusted_data, seed)
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
        test_ints!(
            u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize
        );
    }

    #[test]
    fn test_strings() {
        let mut encrusted = Encrusted::new(TEST_STRING.to_string(), get_seed());
        assert_ne!(encrusted.data.as_bytes(), TEST_STRING.as_bytes());

        {
            let decrusted = encrusted.decrust();
            assert_eq!(*decrusted, TEST_STRING);
        }

        assert_ne!(encrusted.data.as_bytes(), TEST_STRING.as_bytes());
    }

    #[test]
    fn test_strings_from_encrusted() {
        let seed = get_seed();
        let mut encrust_rng = SmallRng::seed_from_u64(seed);

        let mut encrusted_string = TEST_STRING.to_string();

        // Safety: Testing from_encrusted_data requires pre-encrusted data, which is an unsafe
        // operation. The data will not be available without calling `toggle_encrust` again.
        let mut encrusted = unsafe {
            encrusted_string.toggle_encrust(&mut encrust_rng);
            Encrusted::from_encrusted_data(encrusted_string, seed)
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

        let mut encrusted = Encrusted::new(orig_array, get_seed());
        assert_ne!(encrusted.data, orig_array);

        {
            let decrusted = encrusted.decrust();
            assert_eq!(*decrusted, orig_array);
        }

        assert_ne!(encrusted.data, orig_array);
    }

    #[test]
    fn test_arrays_from_encrusted() {
        let seed = get_seed();
        let mut encrust_rng = SmallRng::seed_from_u64(seed);
        let orig_array: [u8; 45] = [
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44,
        ];

        let mut encrusted_array = orig_array;

        // Safety: Testing from_encrusted_data requires pre-encrusted data, which is an unsafe
        // operation. The data will not be available without calling `toggle_encrust` again.
        let mut encrusted = unsafe {
            encrusted_array.toggle_encrust(&mut encrust_rng);
            Encrusted::from_encrusted_data(encrusted_array, seed)
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

        let mut encrusted = Encrusted::new(orig_vec.clone(), get_seed());
        assert_ne!(encrusted.data, orig_vec);

        {
            let decrusted = encrusted.decrust();
            assert_eq!(*decrusted, orig_vec);
        }

        assert_ne!(encrusted.data, orig_vec);
    }

    #[test]
    fn test_vecs_from_encrusted() {
        let seed = get_seed();
        let mut encrust_rng = SmallRng::seed_from_u64(seed);
        let orig_vec = TEST_STRING.as_bytes().to_vec();

        let mut encrusted_vec = orig_vec.clone();

        // Safety: Testing from_encrusted_data requires pre-encrusted data, which is an unsafe
        // operation. The data will not be available without calling `toggle_encrust` again.
        let mut encrusted = unsafe {
            encrusted_vec.toggle_encrust(&mut encrust_rng);
            Encrusted::from_encrusted_data(encrusted_vec, seed)
        };

        assert_ne!(encrusted.data, orig_vec);

        {
            let decrusted = encrusted.decrust();
            assert_eq!(*decrusted, orig_vec);
        }

        assert_ne!(encrusted.data, orig_vec);
    }

    #[test]
    fn test_reseed() {
        let num = 828_627_825_u64;
        let mut encrusted = Encrusted::new(num, get_seed());
        let orig_seed = encrusted.seed;
        let mut rng = rand::rng();

        encrusted.reseed(rng.next_u64());

        // May fail, but the seed is so large that a collision is highly unlikely if it is selected
        // randomly.
        assert_ne!(encrusted.seed, orig_seed);

        {
            let decrusted = encrusted.decrust();
            assert_eq!(*decrusted, num);
        }
    }

    /// Test to make sure that a previously encrusted object can be decrusted with the current
    /// version of `encrust`.
    #[test]
    fn ensure_encrust_has_not_changed() {
        // Safety: Comparing a `String` with invalid UTF-8 in a test should hopefully at worst crash
        // the test.
        let mut test_string = unsafe {
            Encrusted::from_encrusted_data(
                String::from_utf8_unchecked(
                    [
                        55u8, 10u8, 35u8, 94u8, 130u8, 81u8, 207u8, 225u8, 64u8, 17u8, 143u8, 78u8,
                        95u8, 204u8, 50u8, 183u8, 54u8, 185u8, 59u8, 50u8, 163u8, 122u8, 131u8,
                        136u8, 172u8, 79u8, 17u8, 12u8, 56u8, 64u8, 59u8, 173u8, 102u8, 54u8,
                        184u8, 186u8, 1u8, 246u8, 193u8, 136u8, 220u8, 224u8, 117u8, 144u8, 131u8,
                        65u8, 77u8,
                    ]
                    .to_vec(),
                ),
                #[allow(
                    clippy::unreadable_literal,
                    reason = "Arbitrary number chosen at random with no further meaning."
                )]
                5233902475398815152u64,
            )
        };

        let decrusted_test_string = test_string.decrust();
        assert_eq!(*decrusted_test_string, TEST_STRING);
    }
}
