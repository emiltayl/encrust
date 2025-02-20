#![no_std]
#![no_main]

// https://github.com/rust-lang/rust/issues/106864
extern crate alloc;

use core::panic::PanicInfo;

use encrust::{encrust, encrust_vec, hashbytes, hashstring, hashstring_ci};

#[global_allocator]
static GLOBAL: dlmalloc::GlobalDlmalloc = dlmalloc::GlobalDlmalloc;

#[cfg(target_os = "windows")]
mod native {
    unsafe extern "C" {
        unsafe fn _exit(exit_code: i32) -> !;
        unsafe fn _cputs(s: *const i8) -> i32;
    }

    pub fn exit_process(exit_code: i32) -> ! {
        unsafe { _exit(exit_code) }
    }

    pub unsafe fn cstring_print(s: &core::ffi::CStr) -> i32 {
        unsafe { _cputs(s.as_ptr()) }
    }
}
#[cfg(not(target_os = "windows"))]
mod native {
    unsafe extern "C" {
        unsafe fn exit(exit_code: i32) -> !;
        unsafe fn puts(s: *const i8) -> i32;
    }

    pub fn exit_process(exit_code: i32) -> ! {
        unsafe { exit(exit_code) }
    }

    pub unsafe fn cstring_print(s: &core::ffi::CStr) -> i32 {
        unsafe { puts(s.as_ptr()) }
    }
}

use native::*;

// Needed to make code compile on Linux.
// Copied from https://github.com/rust-lang/rust/issues/106864#issuecomment-1858861750
#[unsafe(no_mangle)]
extern "C" fn rust_eh_personality() {}

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
extern "C" fn _Unwind_Resume() {}

#[panic_handler]
fn panic(panic_info: &PanicInfo) -> ! {
    let string = alloc::format!("{panic_info}\n");
    let cstring = alloc::ffi::CString::new(string).unwrap_or_else(|_| {
        let bytes = b"Unable to convert panic message to CString!\n\0";
        unsafe { alloc::ffi::CString::from_vec_with_nul_unchecked(alloc::vec::Vec::from(bytes)) }
    });

    unsafe {
        cstring_print(&cstring);
    }
    exit_process(1)
}

#[unsafe(no_mangle)]
pub extern "C" fn main() -> ! {
    let mut s = encrust!("Hi!");
    let mut n = encrust!([1u8, 2u8, 3u8]);
    let mut v = encrust_vec![3i8, 2i8, 1i8, 0i8];
    let hs = hashstring!("Hi!");
    let hsci = hashstring_ci!("hi!");
    let hb = hashbytes!([1, 2, 3]);
    {
        let decrusted = s.decrust();
        assert_eq!("Hi!".as_bytes(), decrusted.as_bytes());
    }
    {
        let decrusted = n.decrust();
        assert_eq!(&[1u8, 2u8, 3u8], decrusted.as_slice())
    }
    {
        let decrusted = v.decrust();
        assert_eq!(&[3, 2, 1, 0], decrusted.as_slice());
    }

    assert!(hs == "Hi!");
    assert!(hsci == "Hi!");
    assert!(hb == &[1, 2, 3]);

    exit_process(0)
}
