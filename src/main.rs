#![feature(random)]
#[allow(dead_code)]
use std::path::Path;
use std::random;

use banana_cask::Cask;

fn random_bytes_v(len: usize) -> Vec<u8> {
    Vec::from_iter(
        (0..len)
            .into_iter()
            .map(|_| (random::random::<u8>() % 10 + 48)),
    )
}
fn random_bytes_k(len: usize) -> Vec<u8> {
    Vec::from_iter(
        (0..len)
            .into_iter()
            .map(|_| (random::random::<u8>() % 25 + 65)),
    )
}

//48+80=128
//16+64=80
fn main() {
    let mut c = Cask::open(Path::new("testdir"), 1024 * 1024).unwrap();
    (0..1_000_000).for_each(|_| {
        c.put(&random_bytes_k(2).to_ascii_lowercase(), &random_bytes_v(14))
            .unwrap();
    });
}
