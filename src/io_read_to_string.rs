use std::io::{Read, Result};


/// Copy of std::io::read_to_string without unstable marker
pub fn io_read_to_string<R: Read>(mut reader: R) -> Result<String> {
    let mut buf = String::new();
    reader.read_to_string(&mut buf)?;
    Ok(buf)
}

