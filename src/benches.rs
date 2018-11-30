#![cfg(test)]

#[macro_use]
extern crate bencher;

extern crate rust_mdbm;

use rust_mdbm as mdbm;
use std::fs::remove_file;
use std::path::Path;

use bencher::Bencher;

fn bench_set(b: &mut Bencher) {
    let path = Path::new("test_bench_set.db");
    let db = mdbm::MDBM::new(path, mdbm::MDBM_O_RDWR | mdbm::MDBM_O_CREAT, 0o644, 0, 0).unwrap();

    b.iter(|| {
        db.set(&"hello", &"world", 0).unwrap();
    });

    let _ = remove_file(path);
}

fn bench_get(b: &mut Bencher) {
    let path = Path::new("test_bench_get.db");
    let db = mdbm::MDBM::new(path, mdbm::MDBM_O_RDWR | mdbm::MDBM_O_CREAT, 0o644, 0, 0).unwrap();

    db.set(&"hello", &"world", 0).unwrap();

    b.iter(|| {
        let key = "hello";
        let value = db.lock(&key, 0).unwrap();
        let _ = value.get().unwrap();
    });

    let _ = remove_file(path);
}

fn bench_set_get(b: &mut Bencher) {
    let path = Path::new("test_bench_get_set.db");
    let db = mdbm::MDBM::new(path, mdbm::MDBM_O_RDWR | mdbm::MDBM_O_CREAT, 0o644, 0, 0).unwrap();

    b.iter(|| {
        db.set(&"hello", &"world", 0).unwrap();
        let key = "hello";
        let value = db.lock(&key, 0).unwrap();
        let _ = value.get().unwrap();
    });

    let _ = remove_file(path);
}

benchmark_group!(benches, bench_set, bench_get, bench_set_get);
benchmark_main!(benches);
