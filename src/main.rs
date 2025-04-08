#![feature(test)]
#![macro_use]

#[allow(dead_code, unused)]
mod io;

#[macro_export]
macro_rules! s {
    ($s:expr) => {
        $s.to_string()
    };
}

fn main() {
    ()
}
