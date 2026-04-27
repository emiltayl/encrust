# Encrust

[crates-badge]: https://img.shields.io/crates/v/encrust.svg
[crates-url]: https://crates.io/crates/encrust
[docs-image]: https://img.shields.io/docsrs/encrust.svg
[docs-link]: https://docs.rs/encrust/
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://github.com/emiltayl/encrust/blob/main/LICENSE
[build-image]: https://github.com/emiltayl/encrust/actions/workflows/rust.yml/badge.svg?branch=main
[build-link]: https://github.com/emiltayl/encrust/actions/workflows/rust.yml

[![Crates.io][crates-badge]][crates-url]
[![Docs][docs-image]][docs-link]
[![MIT licensed][mit-badge]][mit-url]
[![Build Status][build-image]][build-link]

A Rust crate for obfuscating ("encrusting") data in memory, deobfuscating it only when needed.

Encrust does not provide any security as the seed required to deobfuscate the data is stored right
next to the data itself, no integrity checks are performed. If encrusted bytes are modified,
decrusting may produce invalid values, for example bytes that are not valid UTF-8 for a `String`.

The crate also provides macros for hashing strings and byte arrays at compile time.

## Example
Encrust comes with all features enabled by default. To use, add the following to Cargo.toml:

```toml
[dependencies]
encrust = "0.3"
```

### Encrusting values at compile time

```rust
use encrust::{encrust, encrust_file_string, encrust_file_bytes, encrust_file_cstring};

let mut hidden_string = encrust!("This string will not appear as-is in the executable.");
let mut hidden_number = encrust!(0xabc123u32);
let mut hidden_bytes = encrust!(b"some bytes");

{
    let string = hidden_string.decrust();
    let number = hidden_number.decrust();
    let bytes = hidden_bytes.decrust();

    println!("The string is \"{}\" and the number 0x{:x}.", &*string, *number);
    assert_eq!(b"some bytes", &*bytes);
}

// The guards are now out of scope, so the values are encrusted again.

// It is also possible encrust file contents at compile time. These macros read files relative
// to the calling crate's `CARGO_MANIFEST_DIR` directory.

// Read `Cargo.toml` as a `String` and obfuscate it.
let mut cargo_toml = encrust_file_string!("Cargo.toml");
// Read `Cargo.toml` as an array of `u8` and obfuscate it.
let mut cargo_toml_bytes = encrust_file_bytes!("Cargo.toml");
// Read `Cargo.toml` as a `CString` and obfuscate it.
let mut cargo_toml_cstring = encrust_file_cstring!("Cargo.toml");
```

#### Supported data types

| Data type | Required feature | Decrusted deref target | Example `encrust!` invocation |
| --- | --- | --- | --- |
| `u8`, `i8`, `u16`, `i16`, `u32`, `i32`, `u64`, `i64`, `u128`, `i128`, `usize`, `isize` | None | Same integer type | `encrust!(123u32)` <br> Note that the suffix with the integer type is required. |
| `String` | None | `str` | `encrust!("secret")` |
| `CString` | None | `[u8]` | `encrust!(c"secret")` |
| `[u8; N]` byte string | None | `[u8; N]` | `encrust!(b"secret")` |
| `[T; N] where T: InPlaceEncrust` <br> numeric array | None | `[T; N]` | `encrust!([1u8, 2u8, 3u8])` |
| `[[T; N]; M] where T: InPlaceEncrust` <br> nested numeric array | None | `[[T; N]; M]` | `encrust!([[1u8, 2u8], [3u8, 4u8]])` |

### Encrusting values at run time

```rust
use encrust::Encrusted;

use rand::{rng, Rng};

let mut value = Encrusted::new(String::from("runtime value"), rng().next_u64());

{
    let mut decrusted = value.decrust();
    assert_eq!("runtime value", &*decrusted);
    decrusted.make_ascii_uppercase();
}

assert_eq!("RUNTIME VALUE", &*value.decrust());
```

### Hashing values at compile time

```rust
use encrust::{hashbytes, hashstring, hashstring_ci};

let hashed_string = hashstring!("This string does not appear in the executable");
assert!(hashed_string == "This string does not appear in the executable");

let hashed_string_ci = hashstring_ci!("Case does not matter");
assert!(hashed_string_ci == "cAsE dOeS nOt MaTtEr");

let hashed_bytes = hashbytes!([1, 2, 3, 4]);
assert!(hashed_bytes == &[1, 2, 3, 4]);
```

## Feature flags

Encrust has the following feature flags, all enabled by default:

* `hashstrings`: Hash strings and byte arrays so they can be compared without storing the original
  strings or bytes in the executable.
* `macros`: Include proc macros for encrusting values and hashing literals at compile time.

## License

This project is licensed under the [MIT license].

[MIT license]: https://github.com/emiltayl/encrust/blob/master/LICENSE

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in Encrust shall be licensed as MIT, without any additional terms or conditions.
