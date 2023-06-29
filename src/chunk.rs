use anyhow::Result;

use std::fs::File;
use std::io::{Read, Write};

use sha256::digest;

const MAX_FILE_SIZE: usize = 10 * 1024 * 1024; // 10MB;
const WINDOW_SIZE: usize = 1024 * 4; // 4KB
const MIN_FILE_SIZE: usize = 2 * 1024 * 1024; // 2MB

// Takes a reader and chunks it into files
pub fn create_chunks<R: Read>(r: &mut R) -> Result<Vec<(String, File)>> {
    // But the buffer we'll read into is larger that to reduce disk reads
    let mut buffer = vec![0_u8; MAX_FILE_SIZE];
    // The buffer that gets saved to a file, and the number of writes we've
    // written to it thus far
    let mut flush_buffer = vec![0_u8; MAX_FILE_SIZE];
    let mut bytes_to_flush = 0;

    // The window for our rolling sum
    let mut window = Window::<WINDOW_SIZE>::default();

    let mut ret = vec![];

    while let Ok(i) = r.read(&mut buffer) {
        if i == 0 {
            // EOF reached
            break;
        }

        for byte in buffer.iter().take(i) {
            // Push into our flush buffer, increment the size, and calculate the new sum
            flush_buffer.push(*byte);
            bytes_to_flush += 1;
            let sum = window.push_back(*byte);

            if bytes_to_flush < MIN_FILE_SIZE {
                // If it's too small, don't bother with other conditions
                continue;
            }

            // If the sum isn't right, or we haven't run out of buffer,
            // just keep looping
            if sum != 500000 && bytes_to_flush < MAX_FILE_SIZE {
                continue;
            }

            // If we're here, it's time to flush the file to a temp file
            ret.push(flush(&flush_buffer)?);

            // Reset the flush buffer
            flush_buffer = vec![0_u8; MAX_FILE_SIZE];
            bytes_to_flush = 0;
        }
    }

    // If there's anything left in the buffer flush it
    if bytes_to_flush > 0 {
        ret.push(flush(&flush_buffer)?);
    }

    Ok(ret)
}

// Creates a tempfile with the given data
//
// It will produce a hash-named file, and try to see if that file
// already exists. If it does, we'll just reuse it
fn flush(bytes: &[u8]) -> Result<(String, File)> {
    let hash = digest(bytes);
    let path = std::env::temp_dir().join(&hash);
    let path_str = path.clone().into_os_string().into_string().unwrap();

    // Try to open, otherwise create a new file
    let mut f = if let Ok(file) = File::open(&path) {
        return Ok((path_str, file));
    } else {
        File::create(&path)?
    };
    f.write_all(bytes)?;

    Ok((hash, f))
}

use std::collections::LinkedList;

struct Window<const N: usize> {
    sum: u64,
    list: LinkedList<u8>,
}

impl<const N: usize> Window<N> {
    fn default() -> Self {
        Self {
            list: LinkedList::new(),
            sum: 0,
        }
    }

    fn push_back(&mut self, byte: u8) -> u64 {
        self.sum += byte as u64; // Add to our sum, always
        self.list.push_back(byte);

        if self.list.len() > N {
            // If the list hits capacity, remove from the sum the front
            // element
            let val = self.list.pop_front().unwrap();
            self.sum -= val as u64;
        }

        self.sum
    }
}

#[cfg(test)]
mod create_chunks_tests {
    use std::fs::File;
    use std::io::BufReader;

    use super::*;

    // Tests that we get consistent ranges on a sample file of Chloe
    #[test]
    fn chunks_cat() {
        let f = File::open("./test_samples/cat.jpg").unwrap();
        let file_length = f.metadata().unwrap().len();
        let mut r = BufReader::new(f);

        let chunks = create_chunks(&mut r).unwrap();
        // assert_eq!(chunks, []);
    }

    // Tests that we get a consistent chunk of a simple string
    #[test]
    fn chunks_string() {
        let mut bytes: &[u8] =
            b"When Mr Bilbo Baggins of Bag End announced that he would shortly be \
                   celebrating his eleventyifirst birthday with a party of special \
                   magnificence, there was much talk and excitement in Hobbiton.";
        let len = bytes.len();

        let ranges = create_chunks(&mut bytes).unwrap();
        // assert_eq!(ranges, [(0, 193)]);

        // We didn't leave off any bytes
        // assert_eq!(ranges[ranges.len() - 1].1, len);
    }
}
