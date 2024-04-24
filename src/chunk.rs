use std::{
    fmt::{self, Display},
    string::FromUtf8Error,
};

use super::chunk_type::{ChunkType, ChunkTypeDecodeError, CHUNK_TYPE_SIZE};

pub const LEN_SIZE: usize = 4;
pub const CRC_SIZE: usize = 4;
pub const MIN_CHUNK_SIZE: usize = LEN_SIZE + CHUNK_TYPE_SIZE + CRC_SIZE;

const MAX_LEN: usize = std::i32::MAX as usize;

/// A validated PNG chunk. See the PNG Spec for more details
/// http://www.libpng.org/pub/png/spec/1.2/PNG-Structure.html
#[derive(Debug, Clone)]
pub struct Chunk {
    chunk_type: ChunkType,
    data: Vec<u8>,
}

impl Chunk {
    pub fn new(chunk_type: ChunkType, data: Vec<u8>) -> Self {
        if data.len() > MAX_LEN {
            panic!("Data length exceeds specified maximum of 2^31 bytes.");
        }

        Self { chunk_type, data }
    }

    /// The length of the data portion of this chunk.
    pub fn length(&self) -> u32 {
        self.data.len() as u32
    }

    /// The `ChunkType` of this chunk
    pub fn chunk_type(&self) -> &ChunkType {
        &self.chunk_type
    }

    /// The raw data contained in this chunk in bytes
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// The CRC of this chunk
    pub fn crc(&self) -> u32 {
        const HDLC: crc::Crc<u32> = crc::Crc::<u32>::new(&crc::CRC_32_ISO_HDLC);
        let v: Vec<_> = self
            .chunk_type()
            .bytes()
            .iter()
            .cloned()
            .chain(self.data().iter().cloned())
            .collect();
        HDLC.checksum(&v)
    }

    // Returns the data stored in this chunk as a `String`. This function will return an error
    /// if the stored data is not valid UTF-8.
    pub fn data_as_string(&self) -> Result<String, FromUtf8Error> {
        String::from_utf8(self.data.clone())
    }

    /// Returns this chunk as a byte sequences described by the PNG spec.
    /// The following data is included in this byte sequence in order:
    /// 1. Length of the data *(4 bytes)*
    /// 2. Chunk type *(4 bytes)*
    /// 3. The data itself *(`length` bytes)*
    /// 4. The CRC of the chunk type and data *(4 bytes)*
    pub fn as_bytes(&self) -> Vec<u8> {
        self.length()
            .to_be_bytes()
            .iter()
            .cloned()
            .chain(self.chunk_type().bytes().iter().cloned())
            .chain(self.data().iter().cloned())
            .chain(self.crc().to_be_bytes().iter().cloned())
            .collect()
    }
}

// Extracts array with fixed size of 4 from input.
fn segment4(bytes: &[u8]) -> [u8; 4] {
    bytes.try_into().unwrap()
}

#[derive(Debug)]
pub enum ChunkDecodeError {
    InvalidChunkSize(usize),
    DataExceedMaximumLength(usize),
    LengthMismatch {
        data_length: usize,
        given_length: usize,
    },
    CrcMismatch {
        expected_crc: u32,
        given_crc: u32,
    },
    ChunkTypeDecode(ChunkTypeDecodeError),
}

impl Display for ChunkDecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidChunkSize(size) => writeln!(f, "Invalid chunk size: {size}"),
            Self::DataExceedMaximumLength(length) => {
                writeln!(f, "Chunk data exceed maximum length: {length}")
            }
            Self::LengthMismatch {
                data_length,
                given_length,
            } => writeln!(
                f,
                "Data length mismatch: {data_length} (actual) vs {given_length} (given)"
            ),
            Self::CrcMismatch {
                expected_crc,
                given_crc,
            } => writeln!(
                f,
                "CRC mismatch: {expected_crc} (expected) vs {given_crc} (given)"
            ),
            Self::ChunkTypeDecode(err) => writeln!(f, "Chunk type error: {err}"),
        }
    }
}

impl From<ChunkTypeDecodeError> for ChunkDecodeError {
    fn from(err: ChunkTypeDecodeError) -> Self {
        Self::ChunkTypeDecode(err)
    }
}

impl TryFrom<&[u8]> for Chunk {
    type Error = ChunkDecodeError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let size = bytes.len();
        if size < MIN_CHUNK_SIZE {
            return Err(ChunkDecodeError::InvalidChunkSize(size));
        }

        let data_length = u32::from_be_bytes(segment4(&bytes[0..4])) as usize;
        let chunk_type = segment4(&bytes[4..8]);
        let data = &bytes[8..size - 4];
        let crc = u32::from_be_bytes(segment4(&bytes[size - 4..size]));

        if data.len() > MAX_LEN {
            return Err(ChunkDecodeError::DataExceedMaximumLength(data.len()));
        }

        if data.len() != data_length as usize {
            return Err(ChunkDecodeError::LengthMismatch {
                data_length: data.len(),
                given_length: data_length,
            });
        }

        let s = Self {
            chunk_type: ChunkType::try_from(chunk_type)?,
            data: data.to_vec(),
        };

        let calculated_crc = s.crc();
        if calculated_crc != crc {
            return Err(ChunkDecodeError::CrcMismatch {
                expected_crc: calculated_crc,
                given_crc: crc,
            });
        }

        Ok(s)
    }
}

impl Display for Chunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Chunk {{",)?;
        writeln!(f, "  Length: {}", self.length())?;
        writeln!(f, "  Type: {}", self.chunk_type())?;
        writeln!(f, "  Data: {} bytes", self.data().len())?;
        writeln!(f, "  Crc: {}", self.crc())?;
        writeln!(f, "}}",)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct ChunkIter<'a> {
    cur: &'a [u8],
    corrupted: bool,
}

impl<'a> ChunkIter<'a> {
    pub fn new(cur: &'a [u8]) -> Self {
        Self {
            cur,
            corrupted: false,
        }
    }
}

impl<'a> Iterator for ChunkIter<'a> {
    type Item = Result<Chunk, ChunkDecodeError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.corrupted || self.cur.len() == 0 {
            return None;
        }

        let data_length = u32::from_be_bytes(segment4(&self.cur[..LEN_SIZE]));
        let end = MIN_CHUNK_SIZE + data_length as usize;
        if self.cur.len() < end {
            self.corrupted = true;
            return Some(Err(ChunkDecodeError::InvalidChunkSize(self.cur.len())));
        }

        let bytes = &self.cur[0..end];
        self.cur = &self.cur[end..];

        Chunk::try_from(bytes).map_or_else(
            |err| {
                self.corrupted = true;
                Some(Err(err))
            },
            |chunk| Some(Ok(chunk)),
        )
    }
}

#[cfg(test)]
mod iter_tests {
    use super::*;

    fn valid_chunk() -> Vec<u8> {
        let data_length: u32 = 11;
        let chunk_type = "ItEr".as_bytes();
        let message = "Hello World".as_bytes();
        let crc: u32 = 3520753346;

        data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect()
    }

    fn invalid_chunk() -> Vec<u8> {
        let data_length: u32 = 11; // Valid
        let chunk_type = "iter".as_bytes(); // Invalid
        let message = "Hello World".as_bytes();
        let crc: u32 = 1234; // Invalid

        data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect()
    }

    #[test]
    fn test_valid_chunks() {
        let bytes: Vec<u8> = vec![valid_chunk(), valid_chunk(), valid_chunk()]
            .into_iter()
            .flatten()
            .collect();

        let ref_chunk = valid_chunk();
        for chunk in ChunkIter::new(bytes.as_slice()) {
            assert!(chunk.is_ok());
            assert_eq!(chunk.unwrap().as_bytes().as_slice(), ref_chunk.as_slice());
        }
    }

    #[test]
    fn test_invalid_chunks() {
        let bytes: Vec<u8> = vec![valid_chunk(), invalid_chunk(), valid_chunk()]
            .into_iter()
            .flatten()
            .collect();

        let ref_chunk = valid_chunk();
        let mut iter = ChunkIter::new(bytes.as_slice());

        let first = iter.next().unwrap();
        assert!(first.is_ok());
        assert_eq!(first.unwrap().as_bytes().as_slice(), ref_chunk.as_slice());

        let second = iter.next().unwrap();
        assert!(second.is_err());

        let third = iter.next();
        assert!(third.is_none());
    }

    #[test]
    fn test_invalid_length() {
        let bytes = valid_chunk();
        let big_chunk: Vec<u8> = 12345678u32
            .to_be_bytes()
            .into_iter()
            .chain((&bytes[4..]).iter().copied())
            .chain(valid_chunk().into_iter())
            .collect();

        let mut iter = ChunkIter::new(big_chunk.as_slice());

        assert!(iter.next().unwrap().is_err());
        assert!(iter.next().is_none());
    }

    #[test]
    fn test_empty_buffer() {
        assert!(ChunkIter::new(&[]).next().is_none());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn testing_chunk() -> Chunk {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        Chunk::try_from(chunk_data.as_ref()).unwrap()
    }

    #[test]
    fn test_new_chunk() {
        let chunk_type = ChunkType::from_str("RuSt").unwrap();
        let data = "This is where your secret message will be!"
            .as_bytes()
            .to_vec();
        let chunk = Chunk::new(chunk_type, data);
        assert_eq!(chunk.length(), 42);
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_chunk_length() {
        let chunk = testing_chunk();
        assert_eq!(chunk.length(), 42);
    }

    #[test]
    fn test_chunk_type() {
        let chunk = testing_chunk();
        assert_eq!(chunk.chunk_type().to_string(), String::from("RuSt"));
    }

    #[test]
    fn test_chunk_string() {
        let chunk = testing_chunk();
        let chunk_string = chunk.data_as_string().unwrap();
        let expected_chunk_string = String::from("This is where your secret message will be!");
        assert_eq!(chunk_string, expected_chunk_string);
    }

    #[test]
    fn test_chunk_crc() {
        let chunk = testing_chunk();
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_valid_chunk_from_bytes() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk = Chunk::try_from(chunk_data.as_ref()).unwrap();

        let chunk_string = chunk.data_as_string().unwrap();
        let expected_chunk_string = String::from("This is where your secret message will be!");

        assert_eq!(chunk.length(), 42);
        assert_eq!(chunk.chunk_type().to_string(), String::from("RuSt"));
        assert_eq!(chunk_string, expected_chunk_string);
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_invalid_chunk_from_bytes() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656333;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk = Chunk::try_from(chunk_data.as_ref());

        assert!(chunk.is_err());
    }

    #[test]
    pub fn test_chunk_trait_impls() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk: Chunk = TryFrom::try_from(chunk_data.as_ref()).unwrap();

        let _chunk_string = format!("{}", chunk);
    }
}
