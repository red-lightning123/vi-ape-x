use serde::{de::DeserializeOwned, Serialize};
use std::io::{BufReader, BufWriter, Read, Write};
use zstd::{Decoder, Encoder};

pub fn serialize_into<W, T: ?Sized>(writer: W, value: &T) -> bincode::Result<()>
where
    W: Write,
    T: Serialize,
{
    let writer = Encoder::new(writer, 0).unwrap().auto_finish();
    let writer = BufWriter::new(writer);
    bincode::serialize_into(writer, value)
}

pub fn deserialize_from<R, T: ?Sized>(reader: R) -> bincode::Result<T>
where
    R: Read,
    T: DeserializeOwned,
{
    let reader = Decoder::new(reader).unwrap();
    let reader = BufReader::new(reader);
    bincode::deserialize_from(reader)
}
