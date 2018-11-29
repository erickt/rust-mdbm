#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

// extern crate libc;

// type __builtin_va_list = libc::c_void;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

// pub const MDBM_O_RDONLY: libc::c_int = 0x00000000;
// pub const MDBM_O_WRONLY: libc::c_int = 0x00000001;
// pub const MDBM_O_RDWR: libc::c_int = 0x00000002;

// pub const MDBM_O_CREAT: libc::c_int = 0x00000040;
// pub const MDBM_O_TRUNC: libc::c_int = 0x00000200;
// pub const MDBM_O_ASYNC: libc::c_int = 0x00002000;

// pub const MDBM_INSERT: libc::c_int = 0;
// pub const MDBM_REPLACE: libc::c_int = 1;
// pub const MDBM_INSERT_DUP: libc::c_int = 2;
// pub const MDBM_MODIFY: libc::c_int = 3;
