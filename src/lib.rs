use std::{
    collections::HashMap,
    fs::{self, File},
    io::{self, IoSlice, Seek, Write},
    path::{self, Path, PathBuf},
    time::{self, Instant},
};

use xxhash_rust::xxh3::xxh3_128;

pub struct Cask {
    keydir: HashMap<Vec<u8>, KeydirEntry>,
    file_directory: PathBuf,
    files: Vec<PathBuf>,
    current_file: File,
    next_file_counter: u128,
    start_timestamp: time::Instant,
    // Files can exceed this, but after its met/exceeded a new file will be split into
    max_file_size: u64,
}

impl Cask {
    #[must_use]
    pub fn open(dir_path: &path::Path, max_file_size: u64) -> Result<Cask, io::Error> {
        let file_directory = PathBuf::from(dir_path);

        if !file_directory.exists() {
            fs::create_dir(&file_directory)?;
        }

        let mut files = Vec::new();
        for f_result in fs::read_dir(dir_path)? {
            if let Ok(f) = f_result {
                files.push(f.path());
            }
        }

        let current_file;
        let next_file_counter;

        if files.is_empty() {
            let mut filename = file_directory.clone();
            filename.push(get_filename(0));

            current_file = File::options()
                .append(true)
                .read(true)
                .create(true)
                .open(&filename)?;

            files.push(PathBuf::from(&filename));

            next_file_counter = 1;
        } else {
            files.sort_by_key(|pathbuf| get_number_from_filename(pathbuf));

            current_file = File::options()
                .append(true)
                .read(true)
                .open(files.last().unwrap())?;

            next_file_counter = 1 + get_number_from_filename(files.iter().last().unwrap());
        }

        Ok(Cask {
            keydir: HashMap::new(),
            file_directory,
            files,
            current_file,
            next_file_counter,
            /* FIX: all written timestamps should be offset by
             * latest timestamp read from the file at startup
             * Timestamps cant be trusted across starts until then */
            start_timestamp: Instant::now(),
            max_file_size,
        })
    }

    const HEADER_LEN: usize =
        size_of::<u128>() + size_of::<u128>() + size_of::<u64>() + size_of::<u64>();

    pub fn put(&mut self, key: &[u8], value: &[u8]) -> Result<(), io::Error> {
        let to_be_written = (key.len() + value.len() + Cask::HEADER_LEN) as u64;
        if self.current_file.stream_position()? + to_be_written > self.max_file_size {
            let mut filepath = self.file_directory.clone();
            filepath.push(get_filename(self.next_file_counter));
            self.next_file_counter += 1;

            let new_file = File::options()
                .append(true)
                .read(true)
                .create(true)
                .open(&filepath)?;

            self.current_file = new_file;
            self.files.push(filepath);
        }

        let timestamp = Instant::now()
            .duration_since(self.start_timestamp)
            .as_nanos();

        let mut header = [0u8; Cask::HEADER_LEN];
        header[0x00..0x10].copy_from_slice(&xxh3_128(&([key, value].concat())).to_le_bytes());
        header[0x10..0x20].copy_from_slice(&timestamp.to_le_bytes());
        header[0x20..0x28].copy_from_slice(&key.len().to_le_bytes());
        header[0x28..0x30].copy_from_slice(&value.len().to_le_bytes());

        // TODO: handle if this writes less than full entry (i guess)
        let written = self.current_file.write_vectored(&[
            IoSlice::new(&header),
            IoSlice::new(&key),
            IoSlice::new(&value),
        ])?;

        self.keydir.insert(
            key.to_vec(),
            KeydirEntry {
                file_path: self.files.last().unwrap().clone(),
                offset: self.current_file.stream_position()? - written as u64,
                len: written,
                timestamp,
            },
        );

        Ok(())
    }
}

struct KeydirEntry {
    file_path: PathBuf,
    offset: u64,
    len: usize,
    timestamp: u128,
}

fn get_filename(next: u128) -> String {
    format!("d{:032x}.banana", next)
}
fn get_number_from_filename(filename: &Path) -> u128 {
    // TODO: i think i can chain these but idk how
    let filename_str = filename.to_str().expect(&format!(
        "error parsing number from filename - foreign file in storage directory? filename: {:?}",
        filename
    ));

    let s = filename_str.len();
    u128::from_str_radix(&filename_str[s - 7 - 32..s - 7], 16).expect(&format!(
        "error parsing number from filename - foreign file in storage directory? filename: {:?}",
        filename
    ))
}
