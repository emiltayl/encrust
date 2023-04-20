# Encrust
A Rust crate for obfuscating data in memory by encrypting it, decrypting it only when needed.
Encrust does not provide any security as the key and nonce required to decrypt the data is stored right next it.
No integrity checks are performed either, which could lead to safety issues if the encrypted data is modified, for example resulting in `String`s that are not valid UTF-8.

## Example
Encrust comes with all features enabled by default, to use add the following to Cargo.toml:

```toml
[dependencies]
encrust = "0.1"
```

Encrust can then be used to obfuscate data, optionally at compile-time using the built-in macros.

```rust
use encrust::encrust;

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
```

## Feature flags

Encrust has the following feature flags, all enabled by default:

* `std`: Compile with std. Removing this causes the crate to be built as no_std.
* `macros`: Include macros used for Derive macro and proc macros for obfuscating values at compile-time.
* `rand`: Includes functions to obfuscate data at run-time using a `rand::Rng`, as well as a function to change the key and nonce used.

## License

This project is licensed under the [MIT license].

[MIT license]: https://github.com/emiltayl/encrust/blob/master/LICENSE

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in Encrust shall be licensed as MIT, without any additional terms or conditions.