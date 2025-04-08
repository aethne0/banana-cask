#![feature(test)]
#![macro_use]

#[macro_export]
macro_rules! s {
    ($s:expr) => {
        $s.to_string()
    };
}

fn main() {
    println!("yo yo");
}
