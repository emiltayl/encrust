#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(not(test), no_std)]

//! Crate implementing core functionality for `encrust`. See the main crate for documentation.

#[cfg(feature = "hashstrings")]
mod hashstrings;
#[cfg(feature = "hashstrings")]
pub use hashstrings::*;

#[cfg(not(test))]
extern crate core;

extern crate alloc;

use alloc::ffi::CString;
use alloc::string::String;
use alloc::vec::Vec;
use core::ops::{Deref, DerefMut};

use rand::rngs::SmallRng;
use rand::{RngCore, SeedableRng};
use zeroize::Zeroize;

#[doc(hidden)]
pub mod __private {
    /// Used by encrust's macros to refer to `CString` regardless of `std` availability.
    pub use alloc::ffi::CString;
}

/// Container struct for encrust, accepting [`Encrust`] + `Zeroize` types for obfuscation and
/// deobfuscation when needed.
///
/// Care should be taken if `T` has a non-trivial `Drop` implementation, as `T` is not dropped until
/// `zeroize` has been called on it.
pub struct Encrusted<T>
where
    T: Encrust,
    T::Storage: Zeroize,
{
    data: T::Storage,
    seed: u64,
}

impl<T> Encrusted<T>
where
    T: Encrust,
    T::Storage: Zeroize,
{
    /// Accepts [`Encrust`] + `Zeroize` data and obfuscates it using the provided seed.
    pub fn new(data: T, seed: u64) -> Self {
        let mut data = data.to_storage();

        let mut encrust_rng = SmallRng::seed_from_u64(seed);

        <T as Encrust>::toggle_encrust(&mut data, &mut encrust_rng);

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
    pub const unsafe fn from_encrusted_data(data: T::Storage, seed: u64) -> Self {
        Self { data, seed }
    }

    /// Changes the seed used to obfuscate the underlying data.
    pub fn reseed(&mut self, new_seed: u64) {
        {
            let mut decruster = SmallRng::seed_from_u64(self.seed);

            <T as Encrust>::toggle_encrust(&mut self.data, &mut decruster);
        }

        self.seed = new_seed;

        let mut encrust_rng = SmallRng::seed_from_u64(self.seed);

        <T as Encrust>::toggle_encrust(&mut self.data, &mut encrust_rng);
    }

    /// Deobfuscates the data contained in [`Encrusted`] and returns a [`DecrustGuard`] object that
    /// can be used to access and modify the actual data.
    #[doc(alias("expose", "unlock"))]
    pub fn decrust(&mut self) -> DecrustGuard<'_, T> {
        DecrustGuard::new(self)
    }
}

impl<T> Drop for Encrusted<T>
where
    T: Encrust,
    T::Storage: Zeroize,
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

/// Type used to access encrusted data. Use [`Encrusted::decrust`] to create `DecrustGuard` data.
///
/// When the `DecrustGuard` object is dropped, the underlying data is re-obfuscated.
pub struct DecrustGuard<'decrusted, T>
where
    T: Encrust,
    T::Storage: Zeroize,
{
    encrusted_data: &'decrusted mut Encrusted<T>,
}

impl<'decrusted, T> DecrustGuard<'decrusted, T>
where
    T: Encrust,
    T::Storage: Zeroize,
{
    fn new(encrusted_data: &'decrusted mut Encrusted<T>) -> Self {
        let mut decruster = SmallRng::seed_from_u64(encrusted_data.seed);

        <T as Encrust>::toggle_encrust(&mut encrusted_data.data, &mut decruster);

        Self { encrusted_data }
    }
}

impl<T> Drop for DecrustGuard<'_, T>
where
    T: Encrust,
    T::Storage: Zeroize,
{
    fn drop(&mut self) {
        let mut encrust_rng = SmallRng::seed_from_u64(self.encrusted_data.seed);

        <T as Encrust>::toggle_encrust(&mut self.encrusted_data.data, &mut encrust_rng);
    }
}

impl<T> Deref for DecrustGuard<'_, T>
where
    T: Encrust,
    T::Storage: Zeroize,
{
    type Target = T::Ref;

    fn deref(&self) -> &Self::Target {
        // SAFETY: The data in `self.encrusted_data` was deobfuscated by `DecrustGuard::new`.
        unsafe { <T as Encrust>::as_ref(&self.encrusted_data.data) }
    }
}

impl<T> DerefMut for DecrustGuard<'_, T>
where
    T: Encrust,
    T::Storage: Zeroize,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: The data in `self.encrusted_data` was deobfuscated by `DecrustGuard::new`.
        unsafe { <T as Encrust>::as_mut_ref(&mut self.encrusted_data.data) }
    }
}

/// Trait required to use data types with encrust.
///
/// For types where `Storage` and `Ref` are `Self` it is preferable to implement [`InPlaceEncrust`]
/// as it is simpler. This crates has a blanket implementation of `Encrust` for all types that
/// implement [`InPlaceEncrust`].
pub trait Encrust {
    /// The type used to store the encrusted data for the type implementing `Encrust`.
    ///
    /// For simple types where any bit pattern is valid data such as plain integers, `Storage` can
    /// be `Self`. For types with requirements, such as `String` which requires that its data is
    /// valid UTF-8 at all times, `Storage` must be set to an appropriate type. `String`'s
    /// implementation of `Encrust` sets `Storage` to `Vec<u8>`.
    type Storage;

    /// The type used to access encrusted data. This is the type `DecrustGuard` sets as the `Target`
    /// for its `Deref` and `DerefMut` implementations.
    ///
    /// `Vec` sets `Ref` to a slice of the underlying data to prevent accidentally pushing data to
    /// the `Vec` as this may relocate the data and leave a copy of the underlying data in memory.
    /// Similarly, a `String` only provides a `str`.
    type Ref: ?Sized;

    /// Convert `self` to `Self::Storage` for storage in `Encrusted`.
    fn to_storage(self) -> Self::Storage;

    /// Return a reference to `Self::Ref` from `Self::Storage`. This is essentially `DecrustGuard`'s
    /// `Deref` implementation.
    ///
    /// # Safety
    /// This function must never be called when `storage` is encrusted. Calling `as_ref` on
    /// encrusted data may lead to undefined behavior.
    unsafe fn as_ref(storage: &Self::Storage) -> &Self::Ref;

    /// Return a mutable reference to `Self::Ref` from `Self::Storage`. This is essentially
    /// `DecrustGuard`'s `DerefMut` implementation.
    ///
    /// # Safety
    /// This function must never be called when `storage` is encrusted. Calling `as_ref` on
    /// encrusted data may lead to undefined behavior.
    unsafe fn as_mut_ref(storage: &mut Self::Storage) -> &mut Self::Ref;

    /// Called when obfuscating and deobfuscating data.
    ///
    /// This function should only be used by the encrust crate to toggle obfuscation state. Do
    /// **not** call this function manually.
    ///
    /// Calling `toggle_encrust` itself should always be safe. However, calling `as_ref` or
    /// `as_mut_ref` on a value where `toggle_encrust` has been called an odd number of times may
    /// lead to undefined behavior.
    fn toggle_encrust(storage: &mut Self::Storage, encrust_rng: &mut impl RngCore);
}

/// A simpler alternative to [`Encrust`] for types where `Storage` and `Ref` are `Self`.
///
/// This crate implements [`Encrust`] for all types that implement `InPlaceEncrust`.
pub trait InPlaceEncrust {
    /// Called when obfuscating and deobfuscating data.
    ///
    /// `toggle_encrust` must be safe to use, that is, it may not leave `self` in an invalid state
    /// or otherwise make it unsafe to store or access `self`.
    ///
    /// This function should only be used by the encrust crate to toggle obfuscation state. Do
    /// **not** call this function manually.
    fn toggle_encrust(&mut self, encrust_rng: &mut impl RngCore);
}

impl<T> Encrust for T
where
    T: InPlaceEncrust,
{
    type Storage = Self;
    type Ref = Self;

    fn to_storage(self) -> Self::Storage {
        self
    }

    unsafe fn as_ref(storage: &Self::Storage) -> &Self::Ref {
        storage
    }

    unsafe fn as_mut_ref(storage: &mut Self::Storage) -> &mut Self::Ref {
        storage
    }

    fn toggle_encrust(storage: &mut Self::Storage, encrust_rng: &mut impl RngCore) {
        <T as InPlaceEncrust>::toggle_encrust(storage, encrust_rng);
    }
}

macro_rules! encrust_int {
    ( $( $t:ty ),* ) => {
        $(
            impl InPlaceEncrust for $t {
                fn toggle_encrust(&mut self, encrust_rng: &mut impl ::rand::RngCore) {
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

encrust_int!(
    u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize
);

impl Encrust for String {
    type Storage = Vec<u8>;

    type Ref = str;

    fn to_storage(self) -> Self::Storage {
        self.into_bytes()
    }

    unsafe fn as_ref(storage: &Self::Storage) -> &Self::Ref {
        // SAFETY: It is up to the caller to ensure that it is safe to access the storage as a
        // `str`.
        unsafe { str::from_utf8_unchecked(storage) }
    }

    unsafe fn as_mut_ref(storage: &mut Self::Storage) -> &mut Self::Ref {
        // SAFETY: It is up to the caller to ensure that it is safe to access the storage as a
        // `str`.
        unsafe { str::from_utf8_unchecked_mut(storage) }
    }

    fn toggle_encrust(storage: &mut Self::Storage, encrust_rng: &mut impl RngCore) {
        // TODO possibly replace with <Vec<u8> as Encrust>::toggle_encrust(storage, encrust_rng);?
        // Encrusting 16 bytes at a time as a micro-benchmark showed that it was most efficient on
        // the tested x86-64 systems.
        let mut key: [u8; 16] = [0; 16];
        for chunk in storage.chunks_mut(16) {
            encrust_rng.fill_bytes(&mut key);
            for (byte, byte_key) in chunk.iter_mut().zip(key.iter()) {
                *byte ^= byte_key;
            }
        }
    }
}

// TODO byte slice for now?
impl Encrust for CString {
    type Storage = Vec<u8>;
    // Note: it is currently not supported to get
    type Ref = [u8];

    fn to_storage(self) -> Self::Storage {
        self.into_bytes_with_nul()
    }

    unsafe fn as_ref(storage: &Self::Storage) -> &Self::Ref {
        storage.as_ref()
    }

    unsafe fn as_mut_ref(storage: &mut Self::Storage) -> &mut Self::Ref {
        storage.as_mut()
    }

    fn toggle_encrust(storage: &mut Self::Storage, encrust_rng: &mut impl RngCore) {
        // TODO possibly replace with <Vec<u8> as Encrust>::toggle_encrust(storage, encrust_rng);?
        // Encrusting 16 bytes at a time as a micro-benchmark showed that it was most efficient on
        // the tested x86-64 systems.
        let mut key: [u8; 16] = [0; 16];
        for chunk in storage.chunks_mut(16) {
            encrust_rng.fill_bytes(&mut key);
            for (byte, byte_key) in chunk.iter_mut().zip(key.iter()) {
                *byte ^= byte_key;
            }
        }
    }
}

impl<T, const N: usize> Encrust for [T; N]
where
    T: InPlaceEncrust,
{
    type Storage = Self;
    type Ref = [T; N];

    fn to_storage(self) -> Self::Storage {
        self
    }

    unsafe fn as_ref(storage: &Self::Storage) -> &Self::Ref {
        storage
    }

    unsafe fn as_mut_ref(storage: &mut Self::Storage) -> &mut Self::Ref {
        storage
    }

    fn toggle_encrust(storage: &mut Self::Storage, encrust_rng: &mut impl RngCore) {
        for element in storage {
            element.toggle_encrust(encrust_rng);
        }
    }
}

impl<T> Encrust for Vec<T>
where
    T: InPlaceEncrust,
{
    type Storage = Self;
    type Ref = [T];

    fn to_storage(self) -> Self::Storage {
        self
    }

    unsafe fn as_ref(storage: &Self::Storage) -> &Self::Ref {
        storage.as_ref()
    }

    unsafe fn as_mut_ref(storage: &mut Self::Storage) -> &mut Self::Ref {
        storage.as_mut()
    }

    fn toggle_encrust(storage: &mut Self::Storage, encrust_rng: &mut impl RngCore) {
        for element in storage {
            element.toggle_encrust(encrust_rng);
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
        assert_ne!(encrusted.data, TEST_STRING.as_bytes());

        {
            let decrusted = encrusted.decrust();
            assert_eq!(&*decrusted, TEST_STRING);
        }

        assert_ne!(encrusted.data, TEST_STRING.as_bytes());
    }

    #[test]
    fn test_strings_from_encrusted() {
        let seed = get_seed();
        let mut encrust_rng = SmallRng::seed_from_u64(seed);

        let mut encrusted_string = TEST_STRING.to_string().into_bytes();

        // Safety: Testing from_encrusted_data requires pre-encrusted data, which is an unsafe
        // operation. The data will not be available without calling `toggle_encrust` again.
        let mut encrusted = unsafe {
            <String as Encrust>::toggle_encrust(&mut encrusted_string, &mut encrust_rng);
            Encrusted::<String>::from_encrusted_data(encrusted_string, seed)
        };

        assert_ne!(encrusted.data, TEST_STRING.as_bytes());

        {
            let decrusted = encrusted.decrust();
            assert_eq!(&*decrusted, TEST_STRING);
        }

        assert_ne!(encrusted.data, TEST_STRING.as_bytes());
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
            <[u8; 45] as Encrust>::toggle_encrust(&mut encrusted_array, &mut encrust_rng);
            Encrusted::<[u8; 45]>::from_encrusted_data(encrusted_array, seed)
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
            <Vec<u8> as Encrust>::toggle_encrust(&mut encrusted_vec, &mut encrust_rng);
            Encrusted::<Vec<u8>>::from_encrusted_data(encrusted_vec, seed)
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
            Encrusted::<String>::from_encrusted_data(
                vec![
                    55u8, 10u8, 35u8, 94u8, 130u8, 81u8, 207u8, 225u8, 64u8, 17u8, 143u8, 78u8,
                    95u8, 204u8, 50u8, 183u8, 54u8, 185u8, 59u8, 50u8, 163u8, 122u8, 131u8, 136u8,
                    172u8, 79u8, 17u8, 12u8, 56u8, 64u8, 59u8, 173u8, 102u8, 54u8, 184u8, 186u8,
                    1u8, 246u8, 193u8, 136u8, 220u8, 224u8, 117u8, 144u8, 131u8, 65u8, 77u8,
                ],
                #[allow(
                    clippy::unreadable_literal,
                    reason = "Arbitrary number chosen at random with no further meaning."
                )]
                5233902475398815152u64,
            )
        };

        let decrusted_test_string = test_string.decrust();
        assert_eq!(&*decrusted_test_string, TEST_STRING);
    }
}
