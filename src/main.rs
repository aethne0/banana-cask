#![feature(test)]
#![macro_use]

use std::fs;

use io::ReaderWriter;

#[allow(dead_code, unused)]
mod io;

#[macro_export]
macro_rules! s {
    ($s:expr) => {
        $s.to_string()
    };
}

fn main() {
    fs::remove_file("test.bl").unwrap();

    let mut rw = ReaderWriter::new().unwrap();
    let _ = rw.put(b"asd1", b"zxc1");
    let _ = rw.put(b"asd2", b"zxc2");
    let _ = rw.put(b"asd3", b"zxc2");

    let x = rw.get_last(b"asd2").unwrap();
    dbg!(x);
}
