use xxhash_rust::xxh32::xxh32;

use crate::s;

const PAGE_SIZE: usize = 256;
const SEED: u32 = 0xf00f00f0;
const HEADER_LENGTH: usize = 8; // 4(checksum:u32) + 2(keylen:u16) + 2(valuelen:u16)

#[derive(Debug, PartialEq)]
pub struct Pair {
    pub key: Vec<u8>,
    pub value: Vec<u8>,
}

type StoreIOResult<T> = Result<T, String>;

impl Pair {
    pub fn serialize_into(&self, buffer: &mut [u8]) -> StoreIOResult<usize> {
        assert!(PAGE_SIZE >= (4 + 2 + 2 + self.key.len() + self.value.len()));
        assert!(self.key.len() < u16::MAX as usize);
        assert!(self.value.len() < u16::MAX as usize);
        // reserves first 4 bytes for checksum
        assert_eq!(4, size_of::<u32>());

        buffer[4..6].copy_from_slice(&(self.key.len() as u16).to_le_bytes());
        buffer[6..8].copy_from_slice(&(self.value.len() as u16).to_le_bytes());
        buffer[8..8 + self.key.len()].copy_from_slice(&self.key.as_slice());
        buffer[8 + self.key.len()..8 + self.key.len() + self.value.len()]
            .copy_from_slice(&self.value.as_slice());

        let end_pointer = 8 + self.key.len() + self.value.len();
        let checksum = xxh32(&buffer[4..end_pointer], SEED);

        buffer[0..4].copy_from_slice(&checksum.to_le_bytes());

        Ok(end_pointer)
    }

    pub fn deserialize_from(buffer: &[u8]) -> StoreIOResult<(usize, Pair)> {
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

#[cfg(test)]
mod tests;
