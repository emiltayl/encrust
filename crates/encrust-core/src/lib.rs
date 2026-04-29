#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(not(test), no_std)]
#![doc = include_str!("../README.md")]

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
use core::any::TypeId;
use core::ops::{Deref, DerefMut};
use core::slice;

use rand::rngs::Xoshiro256PlusPlus;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use zeroize::Zeroize;

/// Module with symbols needed by encrust's macros to generate working code regardless of whether
/// the code is compiled with `std` or not.
#[doc(hidden)]
pub mod __private {
    pub use alloc::ffi::CString;
    pub use alloc::string::String;
    pub use alloc::vec;
}

/// Container for encrusted data.
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
    /// Encrust `data` using `seed`.
    pub fn new(data: T, seed: u64) -> Self {
        let mut data = data.to_storage();

        let mut encrust_rng = Xoshiro256PlusPlus::seed_from_u64(seed);

        <T as Encrust>::toggle_encrust(&mut data, &mut encrust_rng);

        Self { data, seed }
    }

    /// Create an [`Encrusted`] value from data that has already been encrusted.
    ///
    /// This is used by macros to include pre-encrusted objects in generated code.
    ///
    /// # Safety
    /// `data` must be the result of calling [`Encrust::toggle_encrust`] exactly once on a valid
    /// value of type `T`, using the same algorithm version and a random number generator seeded
    /// with `seed`.
    ///
    /// If the storage does not match `seed`, then [`Encrusted::decrust`] may produce invalid
    /// values. For example, `String` storage may no longer be valid UTF-8, which would make later
    /// `str` access undefined behavior.
    #[doc(hidden)]
    #[cfg(feature = "macros")]
    pub const unsafe fn from_encrusted_data(data: T::Storage, seed: u64) -> Self {
        Self { data, seed }
    }

    /// Change the seed used to encrust the stored data.
    pub fn reseed(&mut self, new_seed: u64) {
        {
            let mut decruster = Xoshiro256PlusPlus::seed_from_u64(self.seed);

            <T as Encrust>::toggle_encrust(&mut self.data, &mut decruster);
        }

        self.seed = new_seed;

        let mut encrust_rng = Xoshiro256PlusPlus::seed_from_u64(self.seed);

        <T as Encrust>::toggle_encrust(&mut self.data, &mut encrust_rng);
    }

    /// Decrust the stored data and return a guard that gives access to it.
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
    /// Zeroize the stored data and seed before dropping them.
    ///
    /// Note that the data is zeroized prior to being dropped, which may cause problems for the drop
    /// implementation of the underlying data.
    fn drop(&mut self) {
        self.data.zeroize();
        self.seed.zeroize();
    }
}

/// Guard used to access decrusted data.
///
/// When the guard is dropped, the underlying data is encrusted again.
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
        let mut decruster = Xoshiro256PlusPlus::seed_from_u64(encrusted_data.seed);

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
        let mut encrust_rng = Xoshiro256PlusPlus::seed_from_u64(self.encrusted_data.seed);

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
        // SAFETY: `DecrustGuard::new` decrusted the storage, and the guard's exclusive borrow
        // prevents any other code from re-encrusting or replacing it before this access.
        unsafe { <T as Encrust>::as_ref(&self.encrusted_data.data) }
    }
}

impl<T> DerefMut for DecrustGuard<'_, T>
where
    T: Encrust,
    T::Storage: Zeroize,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: `DecrustGuard::new` decrusted the storage, and `&mut self` guarantees unique
        // access to the decrusted storage for the returned mutable reference.
        unsafe { <T as Encrust>::as_mut_ref(&mut self.encrusted_data.data) }
    }
}

/// Trait for types that can be stored in [`Encrusted`].
///
/// For types where `Storage` and `Ref` are `Self` it is preferable to implement [`InPlaceEncrust`]
/// as it is simpler. This crate has a blanket implementation of `Encrust` for all types that
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

    /// Return a reference to `Self::Ref` from `Self::Storage`. This is essentially
    /// [`DecrustGuard`]'s `Deref` implementation.
    ///
    /// # Safety
    /// This function must never be called when `storage` is encrusted. Calling `as_ref` on
    /// encrusted data may lead to undefined behavior.
    unsafe fn as_ref(storage: &Self::Storage) -> &Self::Ref;

    /// Return a mutable reference to `Self::Ref` from `Self::Storage`. This is essentially
    /// [`DecrustGuard`]'s `DerefMut` implementation.
    ///
    /// # Safety
    /// This function must never be called when `storage` is encrusted. Calling `as_ref` on
    /// encrusted data may lead to undefined behavior.
    unsafe fn as_mut_ref(storage: &mut Self::Storage) -> &mut Self::Ref;

    /// Toggle data between encrusted and decrusted form.
    ///
    /// This function should only be used by the encrust crate to toggle the encrust state. Do
    /// **not** call this function manually.
    ///
    /// Calling `toggle_encrust` itself must always be safe. However, calling `as_ref` or
    /// `as_mut_ref` on a value where `toggle_encrust` has been called an odd number of times may
    /// lead to undefined behavior.
    fn toggle_encrust(storage: &mut Self::Storage, encrust_rng: &mut impl Rng);
}

/// A simpler alternative to [`Encrust`] for types where `Storage` and `Ref` are `Self`.
///
/// This crate implements [`Encrust`] for all types that implement `InPlaceEncrust`.
///
/// This trait is safe because the blanket [`Encrust`] implementation stores and exposes `Self`
/// directly. Implementations that use only safe Rust cannot create invalid values of `Self`; if an
/// implementation uses unsafe code to do so, that unsafe code is responsible for upholding Rust's
/// validity rules.
pub trait InPlaceEncrust {
    /// Toggle `self` between encrusted and decrusted form.
    ///
    /// `toggle_encrust` must be safe to call for any valid value of `Self`. It may not leave `self`
    /// in an invalid state or otherwise make safe access to `self` undefined behavior. Calling the
    /// method twice with the same RNG and RNG state must restore the original value.
    ///
    /// This function should only be used by the encrust crate to toggle the encrust state. Do
    /// **not** call this function manually.
    fn toggle_encrust(&mut self, encrust_rng: &mut impl Rng);
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

    fn toggle_encrust(storage: &mut Self::Storage, encrust_rng: &mut impl Rng) {
        <T as InPlaceEncrust>::toggle_encrust(storage, encrust_rng);
    }
}

macro_rules! encrust_int {
    ( $( $t:ty ),* ) => {
        $(
            impl InPlaceEncrust for $t {
                fn toggle_encrust(&mut self, encrust_rng: &mut impl Rng) {
                    let mut bytes = self.to_le_bytes();

                    // Using 8 bytes as most numbers that will be used with encrust are (most
                    // likely) 64-bit or smaller.
                    for chunk in bytes.chunks_mut(8) {
                        let key = encrust_rng.next_u64().to_le_bytes();
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

impl<T, const N: usize> InPlaceEncrust for [T; N]
where
    T: InPlaceEncrust + 'static,
{
    fn toggle_encrust(&mut self, encrust_rng: &mut impl Rng) {
        slice_toggle_encrust::<T>(self, encrust_rng);
    }
}

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

    fn toggle_encrust(storage: &mut Self::Storage, encrust_rng: &mut impl Rng) {
        slice_toggle_encrust::<u8>(storage, encrust_rng);
    }
}

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

    fn toggle_encrust(storage: &mut Self::Storage, encrust_rng: &mut impl Rng) {
        slice_toggle_encrust::<u8>(storage, encrust_rng);
    }
}

impl<T> Encrust for Vec<T>
where
    T: InPlaceEncrust + 'static,
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

    fn toggle_encrust(storage: &mut Self::Storage, encrust_rng: &mut impl Rng) {
        slice_toggle_encrust::<T>(storage, encrust_rng);
    }
}

/// Macro for implementing `toggle_encrust` for a slice of integers.
///
/// # Safety
/// Must be called inside an unsafe block because it casts `encrust_slice` to `&mut [$ty]`. The
/// caller must prove that `T::Storage` is exactly `$ty`, so the cast preserves element type,
/// alignment, and length.
macro_rules! toggle_encrust_for_integer {
    ($ty:ty, $type_id:ident, $slice:ident, $rng:ident) => {{
        const ARRAY_SIZE: usize = core::mem::size_of::<u64>() / core::mem::size_of::<$ty>();

        // The following `assert`s should be optimized away. They help ensure the following:
        // * `T::Storage` is `$ty`, needed to make `slice::from_raw_parts_mut` sound.
        // * There are enough bytes in a `u64` for a `[$ty; ARRAY_SIZE]`.
        assert!($type_id == TypeId::of::<$ty>());
        assert!(core::mem::size_of::<u64>() >= core::mem::size_of::<[$ty; ARRAY_SIZE]>());

        let slice = slice::from_raw_parts_mut($slice.as_mut_ptr().cast::<$ty>(), $slice.len());

        for chunk in slice.chunks_mut(ARRAY_SIZE) {
            let key = $rng.next_u64().to_le_bytes();
            let key: [$ty; ARRAY_SIZE] = core::array::from_fn(|i| {
                let start = i * core::mem::size_of::<$ty>();
                <$ty>::from_le_bytes(
                    key[start..start + core::mem::size_of::<$ty>()]
                        .try_into()
                        .unwrap(),
                )
            });

            for (num, key) in chunk.iter_mut().zip(key.iter()) {
                *num ^= key;
            }
        }
    }};
}

/// Helper function that implements encrust for any slice of `Encrust`-able types.
fn slice_toggle_encrust<T>(encrust_slice: &mut [T::Storage], encrust_rng: &mut impl Rng)
where
    T: Encrust,
    T::Storage: 'static,
{
    let type_id: TypeId = TypeId::of::<T::Storage>();

    if type_id == TypeId::of::<u8>() {
        // SAFETY:
        // * The `TypeId` check proves `T::Storage` is `u8`, so the cast preserves element type,
        //   alignment, and length.
        // * The original slice is shadowed, so no aliasing reference is used while the cast slice
        //   exists.
        let encrust_slice = unsafe {
            slice::from_raw_parts_mut(encrust_slice.as_mut_ptr().cast::<u8>(), encrust_slice.len())
        };

        u8_slice_toggle_encrust(encrust_slice, encrust_rng);
    } else if type_id == TypeId::of::<i8>() {
        // SAFETY: The `TypeId` check above proves `T::Storage` is `i8`.
        unsafe {
            toggle_encrust_for_integer!(i8, type_id, encrust_slice, encrust_rng);
        }
    } else if type_id == TypeId::of::<i16>() {
        // SAFETY: The `TypeId` check above proves `T::Storage` is `i16`.
        unsafe {
            toggle_encrust_for_integer!(i16, type_id, encrust_slice, encrust_rng);
        }
    } else if type_id == TypeId::of::<i32>() {
        // SAFETY: The `TypeId` check above proves `T::Storage` is `i32`.
        unsafe {
            toggle_encrust_for_integer!(i32, type_id, encrust_slice, encrust_rng);
        }
    } else if type_id == TypeId::of::<i64>() {
        // SAFETY: The `TypeId` check above proves `T::Storage` is `i64`.
        unsafe {
            toggle_encrust_for_integer!(i64, type_id, encrust_slice, encrust_rng);
        }
    } else if type_id == TypeId::of::<isize>() {
        // SAFETY: The `TypeId` check above proves `T::Storage` is `isize`.
        unsafe {
            toggle_encrust_for_integer!(isize, type_id, encrust_slice, encrust_rng);
        }
    } else if type_id == TypeId::of::<i128>() {
        // SAFETY:
        // * The `TypeId` check proves `T::Storage` is `i128`, so the cast preserves element type,
        //   alignment, and length.
        // * The original slice is shadowed, so no aliasing reference is used while the cast slice
        //   exists.
        let encrust_slice = unsafe {
            slice::from_raw_parts_mut(
                encrust_slice.as_mut_ptr().cast::<i128>(),
                encrust_slice.len(),
            )
        };

        let mut key = [0u8; 16];

        for num in encrust_slice {
            encrust_rng.fill_bytes(&mut key);
            *num ^= i128::from_le_bytes(key);
        }
    } else if type_id == TypeId::of::<u16>() {
        // SAFETY: The `TypeId` check above proves `T::Storage` is `u16`.
        unsafe {
            toggle_encrust_for_integer!(u16, type_id, encrust_slice, encrust_rng);
        }
    } else if type_id == TypeId::of::<u32>() {
        // SAFETY: The `TypeId` check above proves `T::Storage` is `u32`.
        unsafe {
            toggle_encrust_for_integer!(u32, type_id, encrust_slice, encrust_rng);
        }
    } else if type_id == TypeId::of::<u64>() {
        // SAFETY: The `TypeId` check above proves `T::Storage` is `u64`.
        unsafe {
            toggle_encrust_for_integer!(u64, type_id, encrust_slice, encrust_rng);
        }
    } else if type_id == TypeId::of::<usize>() {
        // SAFETY: The `TypeId` check above proves `T::Storage` is `usize`.
        unsafe {
            toggle_encrust_for_integer!(usize, type_id, encrust_slice, encrust_rng);
        }
    } else if type_id == TypeId::of::<u128>() {
        // SAFETY:
        // * The `TypeId` check proves `T::Storage` is `u128`, so the cast preserves element type,
        //   alignment, and length.
        // * The original slice is shadowed, so no aliasing reference is used while the cast slice
        //   exists.
        let encrust_slice = unsafe {
            slice::from_raw_parts_mut(
                encrust_slice.as_mut_ptr().cast::<u128>(),
                encrust_slice.len(),
            )
        };

        let mut key = [0u8; 16];

        for num in encrust_slice {
            encrust_rng.fill_bytes(&mut key);
            *num ^= u128::from_le_bytes(key);
        }
    } else {
        // Fallback to encrust each item separately if we have no special optimization.
        for element in encrust_slice {
            <T as Encrust>::toggle_encrust(element, encrust_rng);
        }
    }
}

/// `toggle_encrust` for `u8` slices.
///
/// This is handled as a special case as it is by far the most likely slice used with encrust, and
/// the crate author wants to avoid increasing the entropy of `u8` slices by too much.
fn u8_slice_toggle_encrust(encrust_slice: &mut [u8], encrust_rng: &mut impl Rng) {
    let mut shuffle_indices: [u8; 16] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];

    for chunk in encrust_slice.chunks_mut(16) {
        let mut key = encrust_rng.next_u64().to_le_bytes();

        // Mask to 3 bits per key byte (8 distinct values) to limit entropy increase.
        for byte in &mut key {
            *byte &= 0b00_01_10_01;
        }

        // Store how many numbers we swap to avoid attempting to access out of bounds in `chunk`.
        let (n_shuffle_indices, is_odd) = if chunk.len() < 16 {
            // If we are on the last chunk with less than 16 bytes, we need to make sure that we
            // only get indices in range while swapping. If we have an odd number of bytes, we do
            // not shuffle the last byte as the current approach can only shuffle pairs of bytes.

            let (shuffle_count, is_odd) = if (chunk.len() % 2) == 0 {
                (chunk.len(), false)
            } else {
                (chunk.len() - 1, true)
            };

            for (i, elem) in shuffle_indices.iter_mut().enumerate().take(shuffle_count) {
                *elem = u8::try_from(i).unwrap();
            }

            shuffle_indices[0..shuffle_count].shuffle(encrust_rng);

            (shuffle_count, is_odd)
        } else {
            shuffle_indices.shuffle(encrust_rng);

            (chunk.len(), false)
        };

        // For each shuffled index pair, transform the chunk: add/subtract key byte, then swap.
        // This order ensures the transformation is idempotent when applied with the same seed.
        for (pair, key) in shuffle_indices[..n_shuffle_indices].chunks(2).zip(key) {
            let i = usize::from(pair[0]);
            let j = usize::from(pair[1]);

            chunk[i] = chunk[i].wrapping_add(key);
            chunk[j] = chunk[j].wrapping_sub(key);
            chunk.swap(i, j);
        }

        if is_odd {
            *chunk.last_mut().unwrap() ^= key.last().unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_STRING: &str = "The quick brown fox jumps over the lazy dog😊";

    const SEED: u64 = 0x2357_bd11_1317_1d1f;

    macro_rules! test_ints {
        ( $( $t:ty => [$($value:expr),+ $(,)?] ),* $(,)? ) => {
            $(
                $(
                    {
                        let mut encrusted = Encrusted::<$t>::new($value, SEED);

                        {
                            let decrusted = encrusted.decrust();
                            assert_eq!(*decrusted, $value);
                        }

                        {
                            let decrusted = encrusted.decrust();
                            assert_eq!(*decrusted, $value);
                        }
                    }
                )+

                #[cfg(feature = "macros")]
                {
                    $(
                        let mut encrust_rng = Xoshiro256PlusPlus::seed_from_u64(SEED);
                        let mut encrusted_data: $t = $value;
                        encrusted_data.toggle_encrust(&mut encrust_rng);

                        // SAFETY: `toggle_encrust` is called before using `encrusted_data` in
                        // `from_encrusted_data`.
                        let mut encrusted = unsafe {
                            Encrusted::<$t>::from_encrusted_data(encrusted_data, SEED)
                        };

                        assert_ne!(encrusted.data, $value);

                        {
                            let decrusted = encrusted.decrust();
                            assert_eq!(*decrusted, $value);
                        }

                        assert_ne!(encrusted.data, $value);
                    )+
                }
            )*
        };
    }

    macro_rules! test_int_arrays {
        ( $( $t:ty ),* ) => {
            $(
                let zero_array: [$t; 9] = [0, 0, 0, 0, 0, 0, 0, 0, 0];
                {
                    let mut encrusted = Encrusted::<[$t; 9]>::new(zero_array, SEED);
                    assert_ne!(encrusted.data, zero_array);

                    {
                        let decrusted = encrusted.decrust();
                        assert_eq!(*decrusted, zero_array);
                    }

                    assert_ne!(encrusted.data, zero_array);
                }

                #[cfg(feature = "macros")]
                {
                    let mut encrust_rng = Xoshiro256PlusPlus::seed_from_u64(SEED);
                    let mut encrusted_data: [$t; 9] = zero_array;
                    encrusted_data.toggle_encrust(&mut encrust_rng);

                    // SAFETY: `toggle_encrust` is called before using `encrusted_data` in
                    // `from_encrusted_data`.
                    let mut encrusted = unsafe {
                        Encrusted::<[$t; 9]>::from_encrusted_data(encrusted_data, SEED)
                    };

                    assert_ne!(encrusted.data, zero_array);

                    {
                        let decrusted = encrusted.decrust();
                        assert_eq!(*decrusted, zero_array);
                    }

                    assert_ne!(encrusted.data, zero_array);
                }
            )*
        };
    }

    #[test]
    fn test_ints() {
        test_ints!(
            u8 => [0u8, 1u8, u8::MAX],
            i8 => [0i8, 1i8, -1i8, i8::MIN, i8::MAX],
            u16 => [0u16, 1u16, u16::MAX],
            i16 => [0i16, 1i16, -1i16, i16::MIN, i16::MAX],
            u32 => [0u32, 1u32, u32::MAX],
            i32 => [0i32, 1i32, -1i32, i32::MIN, i32::MAX],
            u64 => [0u64, 1u64, u64::MAX],
            i64 => [0i64, 1i64, -1i64, i64::MIN, i64::MAX],
            u128 => [0u128, 1u128, u128::MAX],
            i128 => [0i128, 1i128, -1i128, i128::MIN, i128::MAX],
            usize => [0usize, 1usize, usize::MAX],
            isize => [0isize, 1isize, -1isize, isize::MIN, isize::MAX],
        );
    }

    #[test]
    fn test_int_arrays() {
        test_int_arrays!(
            u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize
        );
    }

    #[test]
    fn test_strings() {
        let mut encrusted = Encrusted::new(TEST_STRING.to_owned(), SEED);
        assert_ne!(encrusted.data, TEST_STRING.as_bytes());

        {
            let decrusted = encrusted.decrust();
            assert_eq!(&*decrusted, TEST_STRING);
        }

        assert_ne!(encrusted.data, TEST_STRING.as_bytes());
    }

    #[test]
    fn test_string_lengths_around_u8_chunk_boundaries() {
        for len in [0, 1, 2, 7, 8, 9, 15, 16, 17, 45] {
            let string = "a".repeat(len);
            let mut encrusted = Encrusted::new(string.clone(), SEED);

            {
                let decrusted = encrusted.decrust();
                assert_eq!(&*decrusted, string);
            }

            {
                let decrusted = encrusted.decrust();
                assert_eq!(&*decrusted, string);
            }
        }
    }

    #[cfg(feature = "macros")]
    #[test]
    fn test_strings_from_encrusted() {
        let mut encrust_rng = Xoshiro256PlusPlus::seed_from_u64(SEED);

        let mut encrusted_string = TEST_STRING.to_owned().into_bytes();

        // SAFETY: Testing `from_encrusted_data` requires pre-encrusted data. The storage below is
        // toggled exactly once with the same seed before it is passed to `from_encrusted_data`.
        let mut encrusted = unsafe {
            <String as Encrust>::toggle_encrust(&mut encrusted_string, &mut encrust_rng);
            Encrusted::<String>::from_encrusted_data(encrusted_string, SEED)
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

        let mut encrusted = Encrusted::new(orig_array, SEED);
        assert_ne!(encrusted.data, orig_array);

        {
            let decrusted = encrusted.decrust();
            assert_eq!(*decrusted, orig_array);
        }

        assert_ne!(encrusted.data, orig_array);
    }

    #[test]
    fn test_arrays_from_encrusted() {
        let mut encrust_rng = Xoshiro256PlusPlus::seed_from_u64(SEED);
        let orig_array: [u8; 45] = [
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44,
        ];

        let mut encrusted_array = orig_array;

        // SAFETY: Testing `from_encrusted_data` requires pre-encrusted data. The storage below is
        // toggled exactly once with the same seed before it is passed to `from_encrusted_data`.
        let mut encrusted = unsafe {
            <[u8; 45] as Encrust>::toggle_encrust(&mut encrusted_array, &mut encrust_rng);
            Encrusted::<[u8; 45]>::from_encrusted_data(encrusted_array, SEED)
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

        let mut encrusted = Encrusted::new(orig_vec.clone(), SEED);
        assert_ne!(encrusted.data, orig_vec);

        {
            let decrusted = encrusted.decrust();
            assert_eq!(*decrusted, orig_vec);
        }

        assert_ne!(encrusted.data, orig_vec);
    }

    #[cfg(feature = "macros")]
    #[test]
    fn test_vecs_from_encrusted() {
        let mut encrust_rng = Xoshiro256PlusPlus::seed_from_u64(SEED);
        let orig_vec = TEST_STRING.as_bytes().to_vec();

        let mut encrusted_vec = orig_vec.clone();

        // SAFETY: Testing `from_encrusted_data` requires pre-encrusted data. The storage below is
        // toggled exactly once with the same seed before it is passed to `from_encrusted_data`.
        let mut encrusted = unsafe {
            <Vec<u8> as Encrust>::toggle_encrust(&mut encrusted_vec, &mut encrust_rng);
            Encrusted::<Vec<u8>>::from_encrusted_data(encrusted_vec, SEED)
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
        let mut encrusted = Encrusted::new(num, SEED);
        let new_seed = SEED ^ 0xffff_ffff_ffff_ffff;

        encrusted.reseed(new_seed);

        {
            let decrusted = encrusted.decrust();
            assert_eq!(*decrusted, num);
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq, Zeroize)]
    struct CustomInPlaceEncrust(u32);

    impl InPlaceEncrust for CustomInPlaceEncrust {
        fn toggle_encrust(&mut self, encrust_rng: &mut impl Rng) {
            self.0 ^= encrust_rng.next_u32();
        }
    }

    #[test]
    fn vec_of_custom_type_uses_fallback_slice_path() {
        let values = vec![
            CustomInPlaceEncrust(0),
            CustomInPlaceEncrust(1),
            CustomInPlaceEncrust(u32::MAX),
            CustomInPlaceEncrust(0xfeed_beef),
        ];

        let values_clone = values.clone();

        let mut encrusted = Encrusted::new(values, SEED);

        assert_ne!(encrusted.data, values_clone);

        let decrusted = encrusted.decrust();
        assert_eq!(&*decrusted, &values_clone);
    }

    /// Test to make sure that a previously encrusted object can be decrusted with the current
    /// version of `encrust`.
    #[cfg(feature = "macros")]
    #[test]
    fn ensure_encrust_has_not_changed() {
        // SAFETY: This storage was captured from a valid `String` encrusted once with the matching
        // seed. It is only accessed after `decrust` restores the valid UTF-8 bytes.
        let mut test_string = unsafe {
            Encrusted::<String>::from_encrusted_data(
                vec![
                    114u8, 87u8, 102u8, 107u8, 117u8, 113u8, 32u8, 110u8, 32u8, 97u8, 33u8, 84u8,
                    128u8, 118u8, 99u8, 105u8, 112u8, 92u8, 110u8, 106u8, 32u8, 120u8, 128u8,
                    109u8, 142u8, 87u8, 56u8, 91u8, 124u8, 16u8, 130u8, 93u8, 119u8, 84u8, 24u8,
                    125u8, 91u8, 121u8, 122u8, 40u8, 106u8, 96u8, 161u8, 175u8, 224u8, 94u8, 146u8,
                ],
                #[allow(
                    clippy::unreadable_literal,
                    reason = "Arbitrary number chosen at random with no further meaning."
                )]
                5233902475398815152u64,
            )
        };

        let decrusted_test_string = test_string.decrust();
        assert_eq!(decrusted_test_string.as_bytes(), TEST_STRING.as_bytes());
    }
}
