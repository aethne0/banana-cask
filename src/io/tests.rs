use super::*;

extern crate test;

#[cfg(test)]
fn round_trip(original: Pair) {
    let mut buffer = [0u8; PAGE_SIZE];
    let s_len = original.serialize_into(&mut buffer).unwrap();
    let (d_len, deserialized) = Pair::deserialize_from(&buffer).unwrap();

    assert_eq!(original, deserialized);
}

#[test]
fn ser_der_1() {
    round_trip(Pair {
        key: b"woohoohaha".to_vec(),
        value: b"major values here brah".to_vec(),
    });
}

#[test]
fn ser_der_2() {
    round_trip(Pair {
        key: b"".to_vec(),
        value: b"".to_vec(),
    });
}

#[test]
fn ser_der_3() {
    const LARGEST_POSSIBLE: usize = PAGE_SIZE - HEADER_LENGTH;
    let key = b"key".to_vec(); // len=3
    let value = [0xff_u8; LARGEST_POSSIBLE - 3].to_vec();
    round_trip(Pair { key, value });
}

#[test]
fn ser_der_4() {
    round_trip(Pair {
        key: b"ez".to_vec(),
        value: b"pz.".to_vec(),
    });
}

#[test]
#[should_panic]
fn ser_der_5() {
    const LARGEST_POSSIBLE: usize = PAGE_SIZE - HEADER_LENGTH;
    let key = b"key".to_vec(); // len=3
    let value = [0xff_u8; LARGEST_POSSIBLE - 3 + 1].to_vec();
    round_trip(Pair { key, value });
}

/**************
 * Benchmarks *
 **************/

#[bench]
fn bench_serialize_zero(b: &mut test::Bencher) {
    let mut buffer = [0u8; PAGE_SIZE];
    let key = b"".to_vec();
    let value = b"".to_vec();
    let p = Pair { key, value };

    b.iter(|| {
        p.serialize_into(&mut buffer);
    });
}

#[bench]
fn bench_serialize_small(b: &mut test::Bencher) {
    let mut buffer = [0u8; PAGE_SIZE];
    let key = b"1".to_vec();
    let value = b"2".to_vec();
    let p = Pair { key, value };

    b.iter(|| {
        p.serialize_into(&mut buffer);
    });
}

#[bench]
fn bench_serialize_avg(b: &mut test::Bencher) {
    let mut buffer = [0u8; PAGE_SIZE];
    let p = Pair {
        key: b"88888888888888888888".to_vec(),
        value: b"8888888888888888888888888888888888888888888888888888888888888888".to_vec(),
    };

    b.iter(|| {
        p.serialize_into(&mut buffer);
    });
}

#[bench]
fn bench_serialize_large(b: &mut test::Bencher) {
    let mut buffer = [0u8; PAGE_SIZE];
    const LARGEST_POSSIBLE: usize = PAGE_SIZE - HEADER_LENGTH;
    let key = b"key".to_vec(); // len=3
    let value = [0xff_u8; LARGEST_POSSIBLE - 3].to_vec();
    let p = Pair { key, value };

    b.iter(|| {
        p.serialize_into(&mut buffer);
    });
}

#[bench]
fn bench_deserialize_zero(b: &mut test::Bencher) {
    let mut buffer = [0u8; PAGE_SIZE];
    const LARGEST_POSSIBLE: usize = PAGE_SIZE - HEADER_LENGTH;
    let key = b"".to_vec();
    let value = b"".to_vec();
    let p = Pair { key, value };
    p.serialize_into(&mut buffer);

    b.iter(|| {
        let _ = Pair::deserialize_from(&buffer);
    });
}

#[bench]
fn bench_deserialize_small(b: &mut test::Bencher) {
    let mut buffer = [0u8; PAGE_SIZE];
    const LARGEST_POSSIBLE: usize = PAGE_SIZE - HEADER_LENGTH;
    let key = b"1".to_vec();
    let value = b"2".to_vec();
    let p = Pair { key, value };
    p.serialize_into(&mut buffer);

    b.iter(|| {
        let _ = Pair::deserialize_from(&buffer);
    });
}

#[bench]
fn bench_deserialize_mid(b: &mut test::Bencher) {
    let mut buffer = [0u8; PAGE_SIZE];
    let p = Pair {
        key: b"88888888888888888888".to_vec(),
        value: b"8888888888888888888888888888888888888888888888888888888888888888".to_vec(),
    };
    p.serialize_into(&mut buffer);

    b.iter(|| {
        let _ = Pair::deserialize_from(&buffer);
    });
}

#[bench]
fn bench_deserialize_large(b: &mut test::Bencher) {
    let mut buffer = [0u8; PAGE_SIZE];
    const LARGEST_POSSIBLE: usize = PAGE_SIZE - HEADER_LENGTH;
    let key = b"key".to_vec(); // len=3
    let value = [0xff_u8; LARGEST_POSSIBLE - 3].to_vec();
    let p = Pair { key, value };
    p.serialize_into(&mut buffer);

    b.iter(|| {
        let _ = Pair::deserialize_from(&buffer);
    });
}
