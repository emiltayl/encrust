# Encrust

Hide data at run time by obfuscating ("encrusting") it when it is not in use.

Encrust obfuscates the underlying data, and only exposes it when needed. This crate does not provide
any security as the seed needed to decrust the data is stored next to the data, and no integrity
checks are performed.

The crate also provides macros for hashing strings and byte arrays at compile time, so the original
strings or bytes do not need to be included in the executable. The hash macros require the
`hashstrings` feature, which is enabled by default.

## Macros

Encrust contains several macros for embedding encrusted values in executables. Encrusting happens at
compile time, and the plain values are not included in the binary.

```rust
use encrust::{encrust, encrust_file_bytes, encrust_file_cstring, encrust_file_string};

let mut encrusted_int = encrust!(1u32);
assert_eq!(*encrusted_int.decrust(), 1u32);

let mut encrusted_string = encrust!("Strings can also be encrusted.");
assert_eq!("Strings can also be encrusted.", &*encrusted_string.decrust());

let mut encrusted_array = encrust!([1u8, 2u8, 3u8]);
assert_eq!(&[1u8, 2u8, 3u8], &*encrusted_array.decrust());

// Read `Cargo.toml` into a `String`, array of `u8`, and `CString` and encrust.
let mut cargo_toml = encrust_file_string!("Cargo.toml");
let mut cargo_toml_bytes = encrust_file_bytes!("Cargo.toml");
let mut cargo_toml_cstring = encrust_file_cstring!("Cargo.toml");
```

### Supported data types

| Data type | Decrusted deref target | Example `encrust!` invocation |
| --- | --- | --- |
| `u8`, `i8`, `u16`, `i16`, `u32`, `i32`, `u64`, `i64`, `u128`, `i128`, `usize`, `isize` | Same integer type | `encrust!(123u32)` <br> Note that the suffix with the integer type is required. |
| `String` | `str` | `encrust!("secret")` |
| `CString` | `[u8]` | `encrust!(c"secret")` |
| `[u8; N]` byte string | `[u8; N]` | `encrust!(b"secret")` |
| `[T; N] where T: InPlaceEncrust` <br> numeric array | `[T; N]` | `encrust!([1u8, 2u8, 3u8])` |
| `[[T; N]; M] where T: InPlaceEncrust` <br> nested numeric array | `[[T; N]; M]` | `encrust!([[1u8, 2u8], [3u8, 4u8]])` |

## Run-time Encrusting

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


### `hashstrings` macros

The `hashstrings` feature contains macros to include hashes of strings and byte arrays without
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
Encrust currently supports only certain simple data structures; most container types are not
supported. Some metadata is not encrusted. For `Vec` and `String`, the buffer contents are
encrusted, but the pointer, length, and capacity fields are not.

Encrusted data is `zeroize`d prior to being dropped. If you need to perform operations with the data
before dropping it, wrap the encrusted data in another type and implement `Drop` for that outer type.
The outer `Drop` implementation can decrust the data before the inner `Encrusted` value is zeroized.

Encrust is intended for obfuscation within one application, not for communicating secrets between
applications. Use a cryptographic protocol for secure communication.

License: MIT
