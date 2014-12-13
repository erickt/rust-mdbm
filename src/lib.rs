#![feature(unsafe_destructor)]

extern crate "mdbm-sys" as mdbm_sys;
extern crate libc;

use std::io::IoError;
use std::mem;
use std::slice;

pub const MDBM_O_RDONLY: uint = mdbm_sys::MDBM_O_RDONLY as uint;
pub const MDBM_O_WRONLY: uint = mdbm_sys::MDBM_O_WRONLY as uint;
pub const MDBM_O_RDWR: uint = mdbm_sys::MDBM_O_RDWR as uint;
pub const MDBM_O_CREAT: uint = mdbm_sys::MDBM_O_CREAT as uint;
pub const MDBM_O_TRUNC: uint = mdbm_sys::MDBM_O_TRUNC as uint;
pub const MDBM_O_ASYNC: uint = mdbm_sys::MDBM_O_ASYNC as uint;

pub struct MDBM {
    db: *mut mdbm_sys::MDBM,
}

impl MDBM {
    /// Open a database.
    pub fn new(
        path: &Path,
        flags: uint,
        mode: uint,
        psize: uint,
        presize: uint
    ) -> Result<MDBM, IoError> {
        unsafe {
            let path = path.to_c_str();
            let db = mdbm_sys::mdbm_open(
                path.as_ptr(),
                flags as libc::c_int,
                mode as libc::c_int,
                psize as libc::c_int,
                presize as libc::c_int);

            if db.is_null() {
                Err(IoError::last_error())
            } else {
                Ok(MDBM { db: db })
            }
        }
    }

    /// Set a key.
    pub fn set<K, V>(&self, key: &K, value: &V, flags: int) -> Result<(), IoError> where
        K: AsDatum,
        V: AsDatum,
    {
        unsafe {
            let rc = mdbm_sys::mdbm_store(
                self.db,
                to_raw_datum(key.as_datum()),
                to_raw_datum(value.as_datum()),
                flags as libc::c_int);

            if rc == -1 {
                Err(IoError::last_error())
            } else {
                Ok(())
            }
        }
    }

    /// Lock a key.
    pub fn lock<'a, K>(&'a self, key: &'a K, flags: int) -> Result<Lock<'a>, IoError> where
        K: AsDatum,
    {
        let rc = unsafe {
            mdbm_sys::mdbm_lock_smart(
                self.db,
                &to_raw_datum(key.as_datum()),
                flags as libc::c_int)
        };

        if rc == 1 {
            Ok(Lock { db: self, key: key.as_datum() })
        } else {
            Err(IoError::last_error())
        }
    }
}

impl Drop for MDBM {
    fn drop(&mut self) {
        unsafe {
            mdbm_sys::mdbm_sync(self.db);
            mdbm_sys::mdbm_close(self.db);
        }
    }
}

pub struct Datum<'a> {
    bytes: &'a [u8],
}

impl<'a> Datum<'a> {
    pub fn new<'a>(bytes: &'a [u8]) -> Datum<'a> {
        Datum { bytes: bytes }
    }
}

pub trait AsDatum for Sized? {
    fn as_datum<'a>(&'a self) -> Datum<'a>;
}

impl<'a, Sized? T: AsDatum> AsDatum for &'a T {
    fn as_datum<'a>(&'a self) -> Datum<'a> { (**self).as_datum() }
}

impl AsDatum for [u8] {
    fn as_datum<'a>(&'a self) -> Datum<'a> {
        Datum::new(self)
    }
}

impl AsDatum for str {
    fn as_datum<'a>(&'a self) -> Datum<'a> {
        self.as_bytes().as_datum()
    }
}

fn to_raw_datum(datum: Datum) -> mdbm_sys::datum {
    mdbm_sys::datum {
        dptr: datum.bytes.as_ptr() as *mut _,
        dsize: datum.bytes.len() as libc::c_int,
    }
}

pub struct Lock<'a> {
    db: &'a MDBM,
    key: Datum<'a>,
}

impl<'a> Lock<'a> {
    /// Fetch a key.
    pub fn get(&self) -> Option<&[u8]> {
        unsafe {
            let value = mdbm_sys::mdbm_fetch(
                self.db.db,
                to_raw_datum(self.key));

            if value.dptr.is_null() {
                None
            } else {
                // we want to constrain the ptr to our lifetime.
                let ptr: &*const u8 = mem::transmute(&value.dptr);
                Some(slice::from_raw_buf(ptr, value.dsize as uint))
            }
        }
    }
}

#[unsafe_destructor]
impl<'a> Drop for Lock<'a> {
    fn drop(&mut self) {
        println!("unlock1");
        unsafe {
            let rc = mdbm_sys::mdbm_unlock_smart(
                self.db.db,
                &to_raw_datum(self.key),
                0);

            assert_eq!(rc, 1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::MDBM;
    use std::str;

    #[test]
    fn test() {
        let db = MDBM::new(
            &Path::new("test.db"),
            super::MDBM_O_RDWR | super::MDBM_O_CREAT,
            0o644,
            0,
            0
        ).unwrap();

        db.set(&"hello", &"world", 0).unwrap();

        {
            // key needs to be an lvalue so the lock can hold a reference to
            // it.
            let key = "hello";

            // Lock the key. RIAA will unlock it when we exit this scope.
            let value = db.lock(&key, 0).unwrap();

            // Convert the value into a string. The lock is still live at this
            // point.
            let value = str::from_utf8(value.get().unwrap()).unwrap();
            assert_eq!(value, "world");
            println!("hello: {}", value);
        }
    }

    // keys can't escape
    /*
    #[test]
    fn test2() {
        let db = MDBM::new(
            &Path::new("test.db"),
            super::MDBM_O_RDWR | super::MDBM_O_CREAT,
            0o644,
            0,
            0
        ).unwrap();

        db.set(&"hello", &"world", 0).unwrap();

        {
            let value = {
                let key = vec![1];
                db.lock(&key.as_slice(), 0).unwrap()
            };

            let value = str::from_utf8(value.get().unwrap()).unwrap();
            assert_eq!(value, "world");
            println!("hello: {}", value);
        }
    }
    */

    /*
    // values can't escape
    #[test]
    fn test3() {
        let _ = {
            let db = MDBM::new(
                &Path::new("test.db"),
                super::MDBM_O_RDWR | super::MDBM_O_CREAT,
                0o644,
                0,
                0
            ).unwrap();

            db.set(&"hello", &"world", 0).unwrap();

            let key = "hello";
            let value = db.lock(&key, 0).unwrap();
            str::from_utf8(value.get().unwrap()).unwrap()
        };
    }
    */
}
