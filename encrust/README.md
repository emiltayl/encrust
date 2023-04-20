# encrust

Hide data at run-time by encrypting it when it is not in use.

Encrust encrypts the underlying data directly, and only exposes the underlying
data when needed. When the decrypted data goes out of scope it is encrypted
until next time it is needed.

## Example usage
```rust
use encrust::{Encrustable, Encrusted};
use zeroize::Zeroize;

// Data types used with encrust must implement Zeroize to make sure data
// does not linger in memory after use.
#[derive(Encrustable, Zeroize)]
struct SecretData (String, u64, Vec<u8>);

// This must be mut, otherwise it is not possible to call decrust.
let mut top_secret = Encrusted::new_with_random(
    SecretData ("A string".to_string(), 1337, vec![1,2,3,4,5,6]),
    rand::thread_rng(),
);

{
    // Decrypt the data in top_secret to be able to read the data.
    let mut decrypted = top_secret.decrust();
    assert_eq!("A string", decrypted.0);
    // It is possible to modify decrypted values as DerefMut is implemented.
    decrypted.1 += 1;
    assert_eq!(1338, decrypted.1);
    assert_eq!(&[1,2,3,4,5,6], decrypted.2.as_slice());
}
// decrypted is now out of scope and the data in top_secret is now encrypted.
```

## Macros
Encrust contains several macros for embedding encrypted values in
executables. Encryption happens at compile-time, and the plain values are
not included in the binary.

```rust
use encrust::{encrust, encrust_vec, encrust_file_bytes, encrust_file_string};

// When encrusting numbers, the data type must be specified.
let mut encrypted_int = encrust!(1u32);
assert_eq!(*encrypted_int.decrust(), 1u32);
let mut encrypted_string = encrust!("Strings can also be encrusted.");
assert_eq!("Strings can also be encrusted.", encrypted_string.decrust().as_str());
let mut encrypted_array = encrust!([1u8,2u8,3u8]);
assert_eq!(&[1u8,2u8,3u8], encrypted_array.decrust().as_slice());
let mut encrypted_vec = encrust_vec![3i32,2i32,1i32];
assert_eq!(vec![3i32,2i32,1i32].as_slice(), encrypted_vec.decrust().as_slice());

// Read Cargo.toml for this crate into a String.
let mut cargo_toml = encrust_file_string!("Cargo.toml");
// Read Cargo.toml for this crate into a byte array.
let mut cargo_toml_bytes = encrust_file_string!("Cargo.toml");
assert_eq!(cargo_toml.decrust().as_bytes(), cargo_toml_bytes.decrust().as_bytes());
```

## Limitations
Encrust currently only offers encryption of certain simple data structures
actually containing data, most container types are not supported yet.
Additionally, certain things are not encrypted at the moment. For vectors
(and strings), the actual data stored is encrypted, but the pointer to the
data, as well as the length and capacity fields are not.

License: MIT
