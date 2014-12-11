#![feature(unsafe_destructor)]

extern crate "mdbm-sys" as mdbm_sys;
extern crate libc;

use std::io::IoError;
use std::mem;
use std::slice;

pub struct MDBM {
    db: *mut mdbm_sys::MDBM,
}

impl MDBM {
    /// Open a database.
    pub fn new(file: &str) -> Result<MDBM, IoError> {
        unsafe {
            let file = file.to_c_str();
            let db = mdbm_sys::mdbm_open(
                file.as_ptr(),
                mdbm_sys::MDBM_O_RDWR | mdbm_sys::MDBM_O_CREAT,
                0o644,
                0,
                0);

            if db.is_null() {
                Err(IoError::last_error())
            } else {
                Ok(MDBM { db: db })
            }
        }
    }

    /// Set a key.
    pub fn set<
        'a,
        K: AsDatum<'a> + 'a,
        V: AsDatum<'a> + 'a,
    >(&self, key: K, value: V, flags: int) -> Result<(), IoError> {
        unsafe {
            let rc = mdbm_sys::mdbm_store(
                self.db,
                key.as_datum().to_raw_datum(),
                value.as_datum().to_raw_datum(),
                flags as libc::c_int);

            if rc == -1 {
                Err(IoError::last_error())
            } else {
                Ok(())
            }
        }
    }

    /// Lock a key.
    pub fn lock<
        'a,
        'b,
        K: AsDatum<'b> + 'b,
    >(&'a self, key: K, flags: int) -> Result<Entry<'a, 'b>, IoError> {
        let key = key.as_datum();
        unsafe {
            let rc = mdbm_sys::mdbm_lock_smart(
                self.db,
                &key.to_raw_datum(),
                flags as libc::c_int);

            if rc == 1 {
                Ok(Entry { db: self, key: key })
            } else {
                Err(IoError::last_error())
            }
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
    pub fn new(bytes: &[u8]) -> Datum {
        Datum { bytes: bytes }
    }

    fn to_raw_datum(&self) -> mdbm_sys::datum {
        mdbm_sys::datum {
            dptr: self.bytes.as_ptr() as *mut _,
            dsize: self.bytes.len() as libc::c_int,
        }
    }
}

pub trait AsDatum<'a> {
    fn as_datum(&self) -> Datum<'a>;
}

impl<'a> AsDatum<'a> for &'a [u8] {
    fn as_datum(&self) -> Datum<'a> {
        Datum::new(*self)
    }
}

impl<'a> AsDatum<'a> for &'a str {
    fn as_datum(&self) -> Datum<'a> {
        Datum::new(self.as_bytes())
    }
}

pub struct Entry<'a, 'b> {
    db: &'a MDBM,
    key: Datum<'b>,
}

impl<'a, 'b> Entry<'a, 'b> {
    /// Fetch a key.
    pub fn get<'c>(&'c self) -> Option<&'c [u8]> {
        unsafe {
            let value = mdbm_sys::mdbm_fetch(
                self.db.db,
                self.key.to_raw_datum());

            if value.dptr.is_null() {
                None
            } else {
                // we want to constrain the ptr to our lifetime.
                let ptr: &'c *const u8 = mem::transmute(&value.dptr);
                Some(slice::from_raw_buf(ptr, value.dsize as uint))
            }
        }
    }
}

#[unsafe_destructor]
impl<'a, 'b> Drop for Entry<'a, 'b> {
    fn drop(&mut self) {
        unsafe {
            let rc = mdbm_sys::mdbm_unlock_smart(
                self.db.db,
                &self.key.to_raw_datum(),
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
        let db = MDBM::new("test.db").unwrap();
        db.set("hello", "world", 0).unwrap();

        {
            let value = db.lock("hello", 0).unwrap();
            let v = str::from_utf8(value.get().unwrap()).unwrap();
            assert_eq!(v, "world");
            println!("hello: {}", v);
        }
    }

    // keys can't escape
    /*
    #[test]
    fn test2() {
        let db = MDBM::new("test.db").unwrap();
        db.set("hello", "world", 0).unwrap();

        {
            let value = {
                let x = vec![1];
                db.lock(x.as_slice(), 0).unwrap()
            };
            let v = str::from_utf8(value.get().unwrap()).unwrap();
            assert_eq!(v, "world");
            println!("hello: {}", v);
        }
    }
    */

    // values can't escape
    /*
    #[test]
    fn test3() {
        let _ = {
            let db = MDBM::new("test.db").unwrap();
            db.set("hello", "world", 0).unwrap();

            let value = db.lock("hello", 0).unwrap();
            str::from_utf8(value.get().unwrap()).unwrap()
        };
    }
    */
}
