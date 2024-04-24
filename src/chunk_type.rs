use std::{
    fmt::{self, Display},
    str::FromStr,
};

pub const CHUNK_TYPE_SIZE: usize = 4;

/// A validated PNG chunk type. See the PNG spec for more details.
/// http://www.libpng.org/pub/png/spec/1.2/PNG-Structure.html
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkType {
    bytes: [u8; CHUNK_TYPE_SIZE],
}

impl ChunkType {
    /// Valid bytes are represented by the characters A-Z or a-z
    fn is_valid_byte(byte: u8) -> bool {
        byte.is_ascii_alphabetic()
    }

    /// Valid bytes are represented by the characters A-Z or a-z
    fn is_valid_bytes(&self) -> bool {
        self.bytes.iter().all(|&b| Self::is_valid_byte(b))
    }

    /// Returns the raw bytes contained in this chunk
    pub fn bytes(&self) -> [u8; 4] {
        self.bytes
    }

    // Returns true if the reserved byte is valid and all four bytes are represented by the characters A-Z or a-z.
    /// Note that this chunk type should always be valid as it is validated during construction.
    pub fn is_valid(&self) -> bool {
        self.is_valid_bytes() && self.is_reserved_bit_valid()
    }

    /// Returns the property state of the first byte as described in the PNG spec
    pub fn is_critical(&self) -> bool {
        self.bytes[0].is_ascii_uppercase()
    }

    /// Returns the property state of the second byte as described in the PNG spec
    pub fn is_public(&self) -> bool {
        self.bytes[1].is_ascii_uppercase()
    }

    /// Returns the property state of the third byte as described in the PNG spec
    pub fn is_reserved_bit_valid(&self) -> bool {
        self.bytes[2].is_ascii_uppercase()
    }

    /// Returns the property state of the fourth byte as described in the PNG spec
    pub fn is_safe_to_copy(&self) -> bool {
        self.bytes[3].is_ascii_lowercase()
    }
}

#[derive(Debug)]
pub enum ChunkTypeDecodeError {
    InputNotAscii([u8; CHUNK_TYPE_SIZE]),
    LengthNot4(usize),
}

impl Display for ChunkTypeDecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InputNotAscii(bytes) => writeln!(f, "Input not ASCII: {:?}", bytes),
            Self::LengthNot4(length) => writeln!(f, "Length of chunk type not 4: {length}"),
        }
    }
}

impl TryFrom<[u8; CHUNK_TYPE_SIZE]> for ChunkType {
    type Error = ChunkTypeDecodeError;

    fn try_from(bytes: [u8; CHUNK_TYPE_SIZE]) -> Result<Self, Self::Error> {
        let c = Self { bytes };
        if !c.is_valid_bytes() {
            Err(ChunkTypeDecodeError::InputNotAscii(bytes))
        } else {
            Ok(Self { bytes })
        }
    }
}

impl FromStr for ChunkType {
    type Err = ChunkTypeDecodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let b = s.as_bytes();
        if b.len() != CHUNK_TYPE_SIZE {
            return Err(Self::Err::LengthNot4(b.len()));
        }

        let b: [u8; CHUNK_TYPE_SIZE] = b[0..4].try_into().unwrap();
        Self::try_from(b)
    }
}

impl Display for ChunkType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(
            f,
            "{}",
            std::str::from_utf8(&self.bytes()).map_err(|_| fmt::Error)?
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;
    use std::str::FromStr;

    #[test]
    pub fn test_chunk_type_from_bytes() {
        let expected = [82, 117, 83, 116];
        let actual = ChunkType::try_from([82, 117, 83, 116]).unwrap();

        assert_eq!(expected, actual.bytes());
    }

    #[test]
    pub fn test_chunk_type_from_str() {
        let expected = ChunkType::try_from([82, 117, 83, 116]).unwrap();
        let actual = ChunkType::from_str("RuSt").unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    pub fn test_chunk_type_is_critical() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_critical());
    }

    #[test]
    pub fn test_chunk_type_is_not_critical() {
        let chunk = ChunkType::from_str("ruSt").unwrap();
        assert!(!chunk.is_critical());
    }

    #[test]
    pub fn test_chunk_type_is_public() {
        let chunk = ChunkType::from_str("RUSt").unwrap();
        assert!(chunk.is_public());
    }

    #[test]
    pub fn test_chunk_type_is_not_public() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(!chunk.is_public());
    }

    #[test]
    pub fn test_chunk_type_is_reserved_bit_valid() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_reserved_bit_valid());
    }

    #[test]
    pub fn test_chunk_type_is_reserved_bit_invalid() {
        let chunk = ChunkType::from_str("Rust").unwrap();
        assert!(!chunk.is_reserved_bit_valid());
    }

    #[test]
    pub fn test_chunk_type_is_safe_to_copy() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_safe_to_copy());
    }

    #[test]
    pub fn test_chunk_type_is_unsafe_to_copy() {
        let chunk = ChunkType::from_str("RuST").unwrap();
        assert!(!chunk.is_safe_to_copy());
    }

    #[test]
    pub fn test_valid_chunk_is_valid() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_valid());
    }

    #[test]
    pub fn test_invalid_chunk_is_valid() {
        let chunk = ChunkType::from_str("Rust").unwrap();
        assert!(!chunk.is_valid());

        let chunk = ChunkType::from_str("Ru1t");
        assert!(chunk.is_err());
    }

    #[test]
    pub fn test_chunk_type_string() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert_eq!(&chunk.to_string(), "RuSt");
    }

    #[test]
    pub fn test_chunk_type_trait_impls() {
        let chunk_type_1: ChunkType = TryFrom::try_from([82, 117, 83, 116]).unwrap();
        let chunk_type_2: ChunkType = FromStr::from_str("RuSt").unwrap();
        let _chunk_string = format!("{}", chunk_type_1);
        let _are_chunks_equal = chunk_type_1 == chunk_type_2;
    }
}
