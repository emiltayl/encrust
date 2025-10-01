# Unreleased

* Set a fixed seed for tests to make the tests deterministic. There are still some randomness in the
  tests, but this *should* not cause test failures in most cases.
* Upgraded `rapidhash` dependency to v4.1.0.

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
