#[cfg(test)]
mod tests;

use std::{
    array,
    collections::HashMap,
    fs::File,
    io::{BufWriter, Read, Seek, SeekFrom, Write},
    iter::Map,
    os::unix::fs::FileExt,
};

use xxhash_rust::xxh32::xxh32;

use crate::s;

const PAGE_SIZE: usize = 256;
const SEED: u32 = 0xf00f00f0;
const HEADER_LENGTH: usize = 8; // 4(checksum:u32) + 2(keylen:u16) + 2(valuelen:u16)
const FILE_PATH: &str = "test.bl";

type StoreIOResult<T> = Result<T, String>;

/*************************************************
 *  file io *
 *************************************************/

#[derive(Debug)]
pub struct ReaderWriter {
    file: File,
    // no concurrency yet
    write_buffer: Box<[u8; PAGE_SIZE]>,
    keydir: HashMap<Vec<u8>, (u64, usize)>,
}

impl ReaderWriter {
    #[must_use]
    pub fn new() -> StoreIOResult<ReaderWriter> {
        match File::options()
            .read(true)
            .append(true)
            .create(true)
            .open(FILE_PATH)
        {
            Ok(file) => {
                let keydir = HashMap::new();

                return Ok(ReaderWriter {
                    file,
                    write_buffer: Box::new([0u8; PAGE_SIZE]),
                    keydir,
                });
            }
            Err(e) => Err(s!(format!("file io error: {:?}", e))),
        }
    }

    pub fn get_last(&self, key: &[u8]) -> StoreIOResult<Option<Vec<u8>>> {
        match self.keydir.get(&key.to_vec()) {
            Some((offset, len)) => {
                let mut buffer = vec![0; *len];
                self.file.read_exact_at(&mut buffer, *offset); // TODO: handle
                let (_len_deser, pair) = Pair::deserialize_from(&buffer).unwrap(); // TODO: handle

                return Ok(Some(pair.value));
            }
            None => Ok(None),
        }
    }

    pub fn put(&mut self, key: &[u8], value: &[u8]) -> StoreIOResult<()> {
        match Pair::serialize_keyval_into(key, value, &mut *self.write_buffer) {
            Ok(len_pair) => {
                let offset = self.file.seek(SeekFrom::Current(0)).unwrap(); // TODO: handle
                match self.file.write(&self.write_buffer[0..len_pair]) {
                    Ok(len_written) => {
                        self.keydir.insert(key.to_vec(), (offset, len_written));
                        return Ok(());
                    }
                    Err(e) => Err(s!(format!("put error: {:?}", e)))?,
                }
                return Ok(());
            }
            Err(e) => Err(s!(format!("put error: {:?}", e))),
        }
    }
}

/*************************************************
 *  key-value pair serialization/deserialization *
 *************************************************/

#[derive(Debug, PartialEq)]
struct Pair {
    pub key: Vec<u8>,
    pub value: Vec<u8>,
}

impl Pair {
    fn serialize_pair_into(&self, buffer: &mut [u8]) -> StoreIOResult<usize> {
        Pair::serialize_keyval_into(self.key.as_slice(), self.value.as_slice(), buffer)
    }

    fn serialize_keyval_into(key: &[u8], value: &[u8], buffer: &mut [u8]) -> StoreIOResult<usize> {
        assert!(PAGE_SIZE >= (4 + 2 + 2 + key.len() + value.len()));
        assert!(key.len() < u16::MAX as usize);
        assert!(value.len() < u16::MAX as usize);
        // reserves first 4 bytes for checksum
        assert_eq!(4, size_of::<u32>());

        buffer[4..6].copy_from_slice(&(key.len() as u16).to_le_bytes());
        buffer[6..8].copy_from_slice(&(value.len() as u16).to_le_bytes());
        buffer[8..8 + key.len()].copy_from_slice(&key);
        buffer[8 + key.len()..8 + key.len() + value.len()].copy_from_slice(&value);

        let end_pointer = 8 + key.len() + value.len();
        let checksum = xxh32(&buffer[4..end_pointer], SEED);

        buffer[0..4].copy_from_slice(&checksum.to_le_bytes());

        Ok(end_pointer)
    }

    fn deserialize_from(buffer: &[u8]) -> StoreIOResult<(usize, Pair)> {
        assert!(buffer.len() >= HEADER_LENGTH);
        let deserialized_checksum = u32::from_le_bytes(buffer[0..4].try_into().unwrap());
        let key_len = u16::from_le_bytes(buffer[4..6].try_into().unwrap()) as usize;
        let val_len = u16::from_le_bytes(buffer[6..8].try_into().unwrap()) as usize;

        let computed_checksum = xxh32(&buffer[4..8 + key_len + val_len], SEED);

        if deserialized_checksum != computed_checksum {
            return Err(s!("checksum mismatch"));
        }

        let key = buffer[8..8 + key_len].to_vec();
        let value = buffer[8 + key_len..8 + key_len + val_len].to_vec();

        Ok((8 + key_len + val_len, Pair { key, value }))
    }
}
