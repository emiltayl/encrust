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

A Rust crate for obfuscating data in memory, deobfuscating it only when needed. Encrust does not
provide any security as the seed required to deobfuscate the data is stored right next to the data
itself. No integrity checks are performed, which could lead to safety issues if the obfuscated data
is modified somehow, for example resulting in `String`s that are not valid UTF-8.

This crate also contains functionality to search for strings or byte arrays without including the
strings or byte arrays in the executable.

## Example
Encrust comes with all features enabled by default. To use, add the following to Cargo.toml:

```toml
[dependencies]
encrust = "0.2"
```

Encrust can then be used to obfuscate data, optionally at compile-time using macros.

```rust
use encrust::{encrust, hashstring};

// Encrust works by directly modifying the underlying memory
// Therefore, encrusted values must be mut in order to be read
let mut hidden_string = encrust!("This string will not appear as-is in the executable.");
// Numbers need to be suffixed wit their data type
let mut hidden_number = encrust!(0xabc123u32);

{
    // "Decrusted" implement Deref and DerefMut to the underlying data
    let string = hidden_string.decrust();
    let number = hidden_number.decrust();

    println!("The string is \"{}\" and the number 0x{:x}.", string.as_str(), *number);
}

// string and number are now out of scope and hidden_string and hidden_number are obfuscated again

use std::io::{self, BufRead};
let hashed_string = hashstring!("This string does not appear in the executable");
let mut line = String::new();
let stdin = std::io::stdin();

println!("Enter the password: ");
stdin.lock().read_line(&mut line).unwrap();

if hashed_string == &line {
  println!("You entered the correct password!");
}
```

## Feature flags

Encrust has the following feature flags, all enabled by default:

* `hashstrings`: Include functionality to hash strings and byte arrays to search for them without
  including the actual strings / bytes in the executable.
* `std`: Compile with std. Removing this causes the crate to be built as no_std.
* `macros`: Include macros used for Derive macro and proc macros for obfuscating values at
  compile-time.

## License

This project is licensed under the [MIT license].

[MIT license]: https://github.com/emiltayl/encrust/blob/master/LICENSE

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in Encrust shall be licensed as MIT, without any additional terms or conditions.