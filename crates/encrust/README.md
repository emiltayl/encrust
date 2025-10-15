# encrust

Hide data at run-time by obfuscating it when it is not in use.

Encrust obfuscates the underlying data, and only exposes it when needed. When the deobfuscated data
goes out of scope it is obfuscated until next time it is needed.

This crate also contains functionality to search for strings or byte patterns without including the
strings / byte patterns themselves in the executable. The functionality requires the `hashstrings`
feature to be enabled, which it is by default.

## Example usage
```rust
use encrust::{Encrustable, Encrusted};
use rand::RngCore;
use zeroize::Zeroize;

// Data types used with encrust must implement Zeroize to make sure data
// does not linger in memory after use.
#[derive(Encrustable, Zeroize)]
struct SecretData (String, u64, Vec<u8>);

let mut rng = rand::rng();

// This must be mut, otherwise it is not possible to call decrust.
let mut top_secret = Encrusted::new(
    SecretData ("A string".to_string(), 1337, vec![1,2,3,4,5,6]),
    rng.next_u64(),
);

{
    // Deobfuscate the data in top_secret to be able to read it.
    let mut deobfuscated = top_secret.decrust();
    assert_eq!("A string", deobfuscated.0);
    // It is possible to modify deobfuscated values as DerefMut is implemented.
    deobfuscated.1 += 1;
    assert_eq!(1338, deobfuscated.1);
    assert_eq!(&[1,2,3,4,5,6], deobfuscated.2.as_slice());
}
// deobfuscated is now out of scope and the data in top_secret is now obfuscated.
```

## Macros
Encrust contains several macros for embedding obfuscated values in executables. Obfuscation happens
at compile-time, and the plain values are not included in the binary.

```rust
use encrust::{encrust, encrust_vec, encrust_file_bytes, encrust_file_string};

// When encrusting numbers, the data type must be specified.
let mut obfuscated_int = encrust!(1u32);
assert_eq!(*obfuscated_int.decrust(), 1u32);
let mut obfuscated_string = encrust!("Strings can also be encrusted.");
assert_eq!("Strings can also be encrusted.", obfuscated_string.decrust().as_str());
let mut obfuscated_array = encrust!([1u8,2u8,3u8]);
assert_eq!(&[1u8,2u8,3u8], obfuscated_array.decrust().as_slice());
let mut obfuscated_vec = encrust_vec![3i32,2i32,1i32];
assert_eq!(vec![3i32,2i32,1i32].as_slice(), obfuscated_vec.decrust().as_slice());

// Read Cargo.toml for this crate into a String.
let mut cargo_toml = encrust_file_string!("Cargo.toml");
// Read Cargo.toml for this crate into a byte array.
let mut cargo_toml_bytes = encrust_file_bytes!("Cargo.toml");
assert!(cargo_toml.decrust().as_bytes() == &cargo_toml_bytes.decrust()[..]);
```

### `hashstrings` macros
The `hashstrings` feature also contains macros to include the hash of strings and byte array without
including the data itself.

```rust
use encrust::{hashstring, hashstring_ci, hashbytes};

let hashed_string = hashstring!("Case sensitive string, hashed");
assert!(hashed_string == "Case sensitive string, hashed");
assert!(hashed_string != "cAsE SeNSItIvE StRinG, HASHED");

let case_insensitive_hashed_string = hashstring_ci!("Case insensitive string, hashed");
assert!(case_insensitive_hashed_string == "Case insensitive string, hashed");
assert!(case_insensitive_hashed_string == "cASe INsenSItiVE StRiNg, hasHed");

let hashed_bytes = hashbytes!([0, 1, 2, 3, 4, 5]);
assert!(hashed_bytes == &[0, 1, 2, 3, 4, 5]);
```

## Limitations
Encrust currently only offers obfuscation of certain simple data structures, most container types
are not supported yet. Additionally, certain data are not obfuscated. For vectors and strings, the
actual data is obfuscated, but pointers to the data as well as the length and capacity fields are
not.

Encrusted data is `zeroize`d prior to being dropped. If you need to perform operations with the data
prior to dropping it, the encrusted data should be wrapped in a struct. The drop logic can then be
implemented for the outermost struct, which can access the encrusted data before it is zeroed.

Encrust is created for obfuscation within an application, and not to communicate secrets between
different applications. A proper cryptographic protocol is recommended if you want secure
communications.

License: MIT
