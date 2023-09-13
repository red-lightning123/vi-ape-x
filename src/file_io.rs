use std::io;
use std::io::{ BufReader, BufWriter };
use std::io::BufRead;
use std::path::Path;
use std::fs::File;

pub fn create_file_buf_write<P : AsRef<Path>>(path : P) -> io::Result<BufWriter<File>> {
    let file = File::create(path)?;
    Ok(BufWriter::new(file))
}

pub fn open_file_buf_read<P : AsRef<Path>>(path : P) -> io::Result<BufReader<File>> {
    let file = File::open(path)?;
    Ok(BufReader::new(file))
}

// directly copied from [https://doc.rust-lang.org/std/io/trait.BufRead.html#method.has_data_left]
// unfortunately, that method isn't stable yet
pub fn has_data_left<R : BufRead>(mut reader : R) -> io::Result<bool> {
    reader.fill_buf().map(|b| !b.is_empty())
}
