use anyhow::Result;

use std::io::Read;

pub fn create_chunks<R: Read>(r: R) -> Result<()> {
    // The sliding window is only 256 bytes
    let mut window: [u8; 256] = [0; 256];
    // But the buffer we'll read into is larger that to reduce disk reads
    let mut buffer: [u8; 1024] = [0; 1024];

    // The loop here will turn the given reader into temp files.
    // We batch reads and writes, so we'll first read into our buffer.
    // After reading into the buffer, we need to add bytes to our window.
    //
    // Then we'll make a hash of the window. If the window doesn't meet chunk
    // requirements (either ending in 00 or accumulated size is 256MB), then add to the write
    // buffer and read the next byte from the buffer and add it to the end of
    // the window.
    //
    // When the write buffer gets full, then we'll call to write to disk.

    Ok(())
}
