#![cfg(test)]

#[macro_use]
extern crate bencher;

extern crate rust_mdbm;

use rust_mdbm as mdbm;
use std::path::Path;

use bencher::Bencher;

fn bench_set(b: &mut Bencher) {
    let db = mdbm::MDBM::new(
        &Path::new("test_bench_set.db"),
        mdbm::MDBM_O_RDWR | mdbm::MDBM_O_CREAT,
        0o644,
        0,
        0,
    )
    .unwrap();

    b.iter(|| {
        db.set(&"hello", &"world", 0).unwrap();
    })
}

fn bench_get(b: &mut Bencher) {
    let db = mdbm::MDBM::new(
        &Path::new("test_bench_get.db"),
        mdbm::MDBM_O_RDWR | mdbm::MDBM_O_CREAT,
        0o644,
        0,
        0,
    )
    .unwrap();

    db.set(&"hello", &"world", 0).unwrap();

    b.iter(|| {
        let key = "hello";
        let value = db.lock(&key, 0).unwrap();
        let _ = value.get().unwrap();
    })
}

fn bench_set_get(b: &mut Bencher) {
    let db = mdbm::MDBM::new(
        &Path::new("test_bench_get_set.db"),
        mdbm::MDBM_O_RDWR | mdbm::MDBM_O_CREAT,
        0o644,
        0,
        0,
    )
    .unwrap();

    b.iter(|| {
        db.set(&"hello", &"world", 0).unwrap();
        let key = "hello";
        let value = db.lock(&key, 0).unwrap();
        let _ = value.get().unwrap();
    })
}

benchmark_group!(benches, bench_set, bench_get, bench_set_get);
benchmark_main!(benches);
