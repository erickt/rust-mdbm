#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

extern crate libc;

type __builtin_va_list = libc::c_void;

include!("ffi.rs")

pub const MDBM_O_RDONLY: libc::c_int = 0x00000000;
pub const MDBM_O_WRONLY: libc::c_int = 0x00000001;
pub const MDBM_O_RDWR: libc::c_int = 0x00000002;

#[cfg(linux)]
pub const MDBM_O_CREAT: libc::c_int = 0x00000040;

#[cfg(not(linux))]
pub const MDBM_O_CREAT: libc::c_int = 0x00000200;
