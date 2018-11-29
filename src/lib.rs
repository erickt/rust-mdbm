#![feature(test)]

extern crate libc;
extern crate test;

use std::io;
use std::mem;
use std::os::unix::ffi::OsStringExt;
use std::slice;

pub const MDBM_O_RDONLY: usize = mdbm_sys::MDBM_O_RDONLY as usize;
pub const MDBM_O_WRONLY: usize = mdbm_sys::MDBM_O_WRONLY as usize;
pub const MDBM_O_RDWR: usize = mdbm_sys::MDBM_O_RDWR as usize;
pub const MDBM_O_CREAT: usize = mdbm_sys::MDBM_O_CREAT as usize;
pub const MDBM_O_TRUNC: usize = mdbm_sys::MDBM_O_TRUNC as usize;
pub const MDBM_O_ASYNC: usize = mdbm_sys::MDBM_O_ASYNC as usize;

pub struct MDBM {
    db: *mut mdbm_sys::MDBM,
}

impl MDBM {
    /// Open a database.
    pub fn new<P: Into<std::path::PathBuf>>(
        path: P,
        flags: usize,
        mode: usize,
        psize: usize,
        presize: usize,
    ) -> Result<MDBM, io::Error> {
        let path_bytes = path.into().into_os_string().into_vec();
        let path_cstring = std::ffi::CString::new(path_bytes)?;

        unsafe{
            let db = mdbm_sys::mdbm_open(
                path_cstring.into_raw(),
                flags as libc::c_int,
                mode as libc::c_int,
                psize as libc::c_int,
                presize as libc::c_int,
            );
        
        if db.is_null() {
            Err(io::Error::last_os_error())
        } else {
            Ok(MDBM { db: db })
        }
        }
    }

    /// Set a key.
    pub fn set<'k, 'v, K, V>(&self, key: &'k K, value: &'v V, flags: isize) -> Result<(), io::Error>
    where
        K: AsDatum<'k> + ?Sized,
        V: AsDatum<'v> + ?Sized,
    {
        unsafe {
            let rc = mdbm_sys::mdbm_store(
                self.db,
                to_raw_datum(&key.as_datum()),
                to_raw_datum(&value.as_datum()),
                flags as libc::c_int,
            );

            if rc == -1 {
                Err(io::Error::last_os_error())
            } else {
                Ok(())
            }
        }
    }

    /// Lock a key.
    pub fn lock<'a, K>(&'a self, key: &'a K, flags: isize) -> Result<Lock<'a>, io::Error>
    where
        K: AsDatum<'a> + ?Sized,
    {
        let rc = unsafe {
            mdbm_sys::mdbm_lock_smart(
                self.db,
                &to_raw_datum(&key.as_datum()),
                flags as libc::c_int,
            )
        };

        if rc == 1 {
            Ok(Lock {
                db: self,
                key: key.as_datum(),
            })
        } else {
            Err(io::Error::last_os_error())
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
    pub fn new(bytes: &'a [u8]) -> Datum<'a> {
        Datum { bytes: bytes }
    }
}

pub trait AsDatum<'a> {
    fn as_datum(&'a self) -> Datum<'a>;
}

impl<'a, T: AsDatum<'a> + ?Sized> AsDatum<'a> for &'a T {
    fn as_datum(&'a self) -> Datum<'a> {
        (**self).as_datum()
    }
}

impl<'a> AsDatum<'a> for [u8] {
    fn as_datum(&'a self) -> Datum<'a> {
        Datum::new(self)
    }
}

impl<'a> AsDatum<'a> for str {
    fn as_datum(&'a self) -> Datum<'a> {
        self.as_bytes().as_datum()
    }
}

fn to_raw_datum(datum: &Datum) -> mdbm_sys::datum {
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
    pub fn get(&'a self) -> Option<&'a [u8]> {
        unsafe {
            let value = mdbm_sys::mdbm_fetch(self.db.db, to_raw_datum(&self.key));

            if value.dptr.is_null() {
                None
            } else {
                // Cast pointer from signed char (c) to unsigned char (rust)
                let u8_ptr: *const u8 = mem::transmute::<*mut i8, *const u8>(value.dptr);
                Some(slice::from_raw_parts(u8_ptr, value.dsize as usize))
            }
        }
    }
}

impl<'a> Drop for Lock<'a> {
    fn drop(&mut self) {
        unsafe {
            let rc = mdbm_sys::mdbm_unlock_smart(self.db.db, &to_raw_datum(&self.key), 0);

            assert_eq!(rc, 1);
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate test;

    use super::MDBM;
    use std::path::Path;
    use std::str;

    #[test]
    fn test_set_get() {
        let db = MDBM::new(
            &Path::new("test.db"),
            super::MDBM_O_RDWR | super::MDBM_O_CREAT,
            0o644,
            0,
            0,
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

    /*
    #[test]
    fn test_keys_cannot_escape() {
        let db = MDBM::new(
            &Path::new("test.db"),
            super::MDBM_O_RDWR | super::MDBM_O_CREAT,
            0o644,
            0,
            0
        ).unwrap();

        db.set(&"hello", &"world", 0).unwrap();

        let _ = {
            let key = vec![1];
            db.lock(&key.as_slice(), 0).unwrap()
        };
    }
    */

    /*
    #[test]
    fn test_values_cannot_escape() {
        let db = MDBM::new(
            &Path::new("test.db"),
            super::MDBM_O_RDWR | super::MDBM_O_CREAT,
            0o644,
            0,
            0
        ).unwrap();

        let _ = {
            db.set(&"hello", &"world", 0).unwrap();

            let key = "hello";
            let value = db.lock(&key, 0).unwrap();
            str::from_utf8(value.get().unwrap()).unwrap()
        };
    }
    */

    /*
    #[test]
    fn test_values_cannot_escape_database() {
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
            db.lock(&key, 0).unwrap();
            str::from_utf8(value.get().unwrap()).unwrap()
        };
    }
    */

    #[bench]
    fn bench_set(b: &mut test::Bencher) {
        let db = MDBM::new(
            &Path::new("test_bench_set.db"),
            super::MDBM_O_RDWR | super::MDBM_O_CREAT,
            0o644,
            0,
            0,
        ).unwrap();

        b.iter(|| {
            db.set(&"hello", &"world", 0).unwrap();
        })
    }

    #[bench]
    fn bench_get(b: &mut test::Bencher) {
        let db = MDBM::new(
            &Path::new("test_bench_get.db"),
            super::MDBM_O_RDWR | super::MDBM_O_CREAT,
            0o644,
            0,
            0,
        ).unwrap();

        db.set(&"hello", &"world", 0).unwrap();

        b.iter(|| {
            let key = "hello";
            let value = db.lock(&key, 0).unwrap();
            let _ = value.get().unwrap();
        })
    }

    #[bench]
    fn bench_set_get(b: &mut test::Bencher) {
        let db = MDBM::new(
            &Path::new("test_bench_get_set.db"),
            super::MDBM_O_RDWR | super::MDBM_O_CREAT,
            0o644,
            0,
            0,
        ).unwrap();

        b.iter(|| {
            db.set(&"hello", &"world", 0).unwrap();
            let key = "hello";
            let value = db.lock(&key, 0).unwrap();
            let _ = value.get().unwrap();
        })
    }
}
