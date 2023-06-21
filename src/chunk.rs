use anyhow::Result;

use std::io::{Read, Seek, SeekFrom};

const WINDOW_SIZE: usize = 1024 * 4;
const MAX_FILE_SIZE: usize = 10 * 1024 * 1024; // 10MB;
const MIN_FILE_SIZE: usize = 2 * 1024 * 1024; // 2MB

pub fn create_chunks<R: Read>(r: &mut R) -> Result<Vec<(usize, usize)>> {
    // But the buffer we'll read into is larger that to reduce disk reads
    let mut buffer = vec![0_u8; MAX_FILE_SIZE];
    let mut window = Window::<WINDOW_SIZE>::default();

    // We use two buffers here:
    // one buffer to read only a piece of the file at a time, and the other is the window
    // for calculating the hash.
    let mut ranges: Vec<(usize, usize)> = vec![];

    let mut current_pos = 0; // The number of buffer's we've grabbed so far

    // We're going to find the different ranges of our file where we want to split
    let mut start = 0; // The first half of the range
    while let Ok(i) = r.read(&mut buffer) {
        if i == 0 {
            // EOF reached
            break;
        }

        for n in 0..i {
            current_pos += 1; // Track out current position
                              // Tack on the byte and figure out if the hash matches our pattern
            let sum = window.push_back(buffer[n]);
            if current_pos - start < MIN_FILE_SIZE && sum == 500000 {
                ranges.push((start, current_pos));
                start = current_pos + 1;
            }
        }

        // Need to use max file size to cap a range
        if current_pos - start == MAX_FILE_SIZE {
            ranges.push((start, current_pos));
            start = current_pos + 1;
        }
    }

    // One final range: whatever was left over
    ranges.push((start, current_pos));

    Ok(ranges)
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

        let ranges = create_chunks(&mut r).unwrap();
        assert_eq!(
            ranges,
            [(0, 1875473), (1875474, 1968804), (1968805, 2289659)]
        );

        // We didn't leave off any bytes
        assert_eq!(ranges[ranges.len() - 1].1 as u64, file_length);
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
        assert_eq!(ranges, [(0, 193)]);

        // We didn't leave off any bytes
        assert_eq!(ranges[ranges.len() - 1].1, len);
    }
}
