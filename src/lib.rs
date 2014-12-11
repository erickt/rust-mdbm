#![feature(unsafe_destructor)]

extern crate "mdbm-sys" as mdbm_sys;
extern crate libc;

use std::slice;
use std::mem;

pub struct MDBM {
    db: *mut mdbm_sys::MDBM,
}

impl MDBM {
    /// Open a database.
    pub fn new(file: &str) -> Result<MDBM, ()> {
        unsafe {
            let file = file.to_c_str();
            let db = mdbm_sys::mdbm_open(
                file.as_ptr(),
                mdbm_sys::MDBM_O_RDWR | mdbm_sys::MDBM_O_CREAT,
                0755,
                0,
                0);

            if db.is_null() {
                Err(())
            } else {
                Ok(MDBM { db: db })
            }
        }
    }

    /// Set a key.
    pub fn set<
        'a,
        K: AsDatum,
        V: AsDatum,
    >(&self, key: K, value: V, flags: int) -> Result<(), ()> {
        let k = key.as_datum();
        let v = value.as_datum();

        unsafe {
            let rc = mdbm_sys::mdbm_store(
                self.db,
                k.datum,
                v.datum,
                flags as libc::c_int);

            if rc == 0 {
                Ok(())
            } else {
                Err(())
            }
        }
    }

    /// Lock a key.
    pub fn lock<
        'a,
        'b,
        K: AsDatum,
    >(&'a self, key: K, flags: int) -> Result<Entry<'a, 'b>, ()> {
        let key = key.as_datum();

        unsafe {
            let rc = mdbm_sys::mdbm_lock_smart(
                self.db,
                &key.datum,
                flags as libc::c_int);

            if rc == 1 {
                Ok(Entry { db: self, key: key })
            } else {
                Err(())
            }
        }
    }
}

impl Drop for MDBM {
    fn drop(&mut self) {
        unsafe {
            mdbm_sys::mdbm_close(self.db);
        }
    }
}

pub struct Datum<'a> {
    datum: mdbm_sys::datum,
}

impl<'a> Datum<'a> {
    pub fn new(v: &[u8]) -> Datum {
        let datum = mdbm_sys::datum {
            dptr: v.as_ptr() as *mut _,
            dsize: v.len() as libc::c_int,
        };
        Datum { datum: datum }
    }
}

pub trait AsDatum {
    fn as_datum<'a>(&'a self) -> Datum<'a>;
}

impl<'a> AsDatum for &'a [u8] {
    fn as_datum<'a>(&'a self) -> Datum<'a> {
        Datum::new(*self)
    }
}

impl<'a> AsDatum for &'a str {
    fn as_datum<'a>(&'a self) -> Datum<'a> {
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
            let value = mdbm_sys::mdbm_fetch(self.db.db, self.key.datum);
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
            let rc = mdbm_sys::mdbm_unlock_smart(self.db.db, &self.key.datum, 0);
            assert_eq!(rc, 0);
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
            println!("hello: {}", str::from_utf8(value.get().unwrap()));
        }
    }
}
