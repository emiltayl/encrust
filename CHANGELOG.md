# Unreleased

# Version 0.4.0 - 2026-04-30

## Changed
* Changed the name of the `Encrustable` trait to `Encrust` to align the name better with usual
  naming of traits in Rust.
* Made major changes to the `Encrust` trait.
  * New types that must be defined:
    * `Storage` - what type `Encrusted` should use for storing the type. This was introduced to
      avoid undefined behavior with `String`s as they require that the data stored in the `String`s
      are valid UTF-8 at all times, even when its not accessed.
    * `Ref` - the type that `DecrustGuard` provides a reference to. This allows restricting access to
      encrusted values. `String` defines `Ref = str` and `Vec<T>` defines `Ref = [T]`. This prevents
      accidentally pushing new data to the values, which could leave an old plaintext copy in memory.
  * New functions that must be defined:
    * `fn to_storage(self) -> Self::Storage` - to prepare a value for storage in `Encrusted`.
    * `unsafe fn as_ref(storage: &Self::Storage) -> &Self::Ref` - convert from a reference to a
      stored value to a reference to `Ref`. Calling `as_ref` on encrusted data may lead to
      undefined behavior.
    * `unsafe fn as_mut_ref(storage: &mut Self::Storage) -> &mut Self::Ref` - same as `as_ref`,
      only with mutable references.
  * `toggle_encrust` is no longer unsafe. It should always be safe to call `toggle_encrust`, but
    accessing an encrusted value through `as_ref` or `as_mut_ref` may lead to undefined behavior if
    the value is encrusted.
  * `Encrust` is no longer implemented for arrays and `Vec`s of types that implement `Encrust`.
    Instead, the arrays and `Vec`s must now be of types that implement the new trait
    `InPlaceEncrust`.
* Renamed `Decrusted` to `DecrustGuard`.
* Changed `toggle_encrust` for `u8` slices. The new approach shuffles 16 byte chunks and modifies
  each byte by one of 8 possible values to decrease entropy gain.
* MSRV changed from 1.85 to 1.87.
* Updated `rand` dependency to version 0.10.1.

## Added
* Introduced a `InPlaceEncrust` trait for simple types such as integers where `Storage` and `Ref`
  can be `Self`. `Encrust` is implemented for all `InPlaceEncrust` types.
* Added `Encrust` implementation for `CString`. `Ref` is currently `[u8]` as there is no good way to
  get an `&mut CStr` from a `Vec<u8>`.
* Added `b""` and `c""` literals to the `encrust!` macro.
* Added `encrust_file_cstring!` macro to read the file contents of a file into a `CString` and
  encrust it at compile time.

## Removed
* Removed the derive macro for `Encrust` as a general derive macro does not work with the changed
  `Encrust` trait.
* Removed the `encrust_vec!` macro as encrusted `Vec`s no longer allow access to the underlying
  `Vec`, but simply a slice to the `Vec`'s memory. With this change, `encrust_vec!([...])` is simply
  a worse alternative to `encrust!([...])` as decrusting does not allow growing the `Vec`'s memory.
* Removed `std` feature as it did not add anything.

# Version 0.3.1 and 0.3.2 - 2025-11-27
* Remove configuration and annotations that are no longer needed to generate documentation.
  Information about required feature flags are now added automatically.
* Added `#[cfg(feature = "hashstrings")]` to types in the `hashstrings` module of `encrust-core`.
  This was done to include information about the required `hashstrings` feature to use types defined
  in the `hashstrings` module.

# Version 0.3.0 - 2025-10-15

* Set a fixed seed for tests to make the tests deterministic. There are still some randomness in the
  tests, but this *should* not cause test failures in most cases.
* Upgraded `rapidhash` dependency to v4.1.0. This prevents depending on two different versions of
  the `rand` crate (0.8 and 0.9).
* Changed the hashing algorithm used by `Hashstring` and `Hashbytes` to rapidhash V3. This means
  that a seed and hash value from encrust 0.2.1 or earlier will not work if using this version, or
  newer, of encrust. New tests have been added to make sure that similar changes are detected
  automatically in the future. Ideally, a change like this should not happen again, but if it does,
  it will be accompanied by a major version bump.

# Version 0.2.1 - 2025-06-18

* Fixed encrust on big endian architectures. [#11]

[#11]: https://github.com/emiltayl/encrust/pull/11

# Version 0.2.0 - 2025-02-20

* Upgraded `rand` dependency to 0.9.0.
* Replaced `XChacha8` with `rand::SmallRng` for obfuscating data.
* New `hashstrings` features to allow searching for strings and bytes without including the data
  itself.
* Fixed macro generation so encrust could actually be used by other crates without depending on both
  `encrust` and `encrust_core`.
* Upgraded Rust edition to 2024 and set the minimum supported rust version to 1.85.
* Removed `rand` feature flag
  * Removed `new_with_random` from `Encrusted` as generating new random values are easier now that
    only a single u64 is needed.
  * Modified the `reseed` function to accept an new seed instead of an RNG to generate a new seed.
