use std::{
    fmt::{self, Display},
    fs::File,
    io::{self, Read, Write},
    path::Path,
    str::FromStr,
};

use crate::{
    chunk::{Chunk, ChunkDecodeError},
    chunk_type::{ChunkType, ChunkTypeDecodeError},
    png::{ChunkNotFoundError, Png, PngDecodeError},
};

#[derive(Debug)]
pub enum CommandError {
    File(io::Error),
    ChunkTypeDecode(ChunkTypeDecodeError),
    ChunkDecode(ChunkDecodeError),
    PngDecode(PngDecodeError),
    ChunkNotFound(ChunkNotFoundError),
}

impl Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::File(err) => writeln!(f, "File error: {err}"),
            Self::ChunkTypeDecode(err) => writeln!(f, "Chunk type decode error: {err}"),
            Self::ChunkDecode(err) => writeln!(f, "Chunk decode error: {err}"),
            Self::PngDecode(err) => writeln!(f, "PNG decode error: {err}"),
            Self::ChunkNotFound(err) => writeln!(f, "{err}"),
        }
    }
}

impl From<io::Error> for CommandError {
    fn from(err: io::Error) -> Self {
        Self::File(err)
    }
}

impl From<ChunkTypeDecodeError> for CommandError {
    fn from(err: ChunkTypeDecodeError) -> Self {
        Self::ChunkTypeDecode(err)
    }
}

impl From<ChunkDecodeError> for CommandError {
    fn from(err: ChunkDecodeError) -> Self {
        Self::ChunkDecode(err)
    }
}

impl From<PngDecodeError> for CommandError {
    fn from(err: PngDecodeError) -> Self {
        Self::PngDecode(err)
    }
}

impl From<ChunkNotFoundError> for CommandError {
    fn from(err: ChunkNotFoundError) -> Self {
        Self::ChunkNotFound(err)
    }
}

fn read_png<P: AsRef<Path>>(file_path: P) -> Result<Png, CommandError> {
    let mut file = File::open(file_path)?;

    let mut buf: Vec<u8> = Vec::new();
    file.read_to_end(&mut buf)?;

    Ok(Png::try_from(buf.as_ref())?)
}

fn save_png<P: AsRef<Path>>(file_path: P, png: Png) -> Result<(), CommandError> {
    let mut file = File::options().truncate(true).write(true).open(file_path)?;

    file.write_all(&png.as_bytes())?;
    Ok(())
}

/// Encodes a message into a PNG file and saves the result
pub fn encode(
    file_path: &str,
    chunk_type: &str,
    message: &str,
    output: Option<&str>,
) -> Result<(), CommandError> {
    let mut png = read_png(file_path)?;
    let chunk_type = ChunkType::from_str(chunk_type)?;
    let chunk = Chunk::new(chunk_type, message.as_bytes().to_vec());

    png.append_chunk(chunk);
    let output = match output {
        Some(o) => o,
        None => file_path,
    };
    save_png(output, png)
}

/// Searches for a message hidden in a PNG file and prints the message if one is found
pub fn decode(file_path: &str, chunk_type: &str) -> Result<(), CommandError> {
    let png = read_png(file_path)?;

    match png.chunk_by_type(chunk_type) {
        Some(chunk) => println!("Message found: {}", chunk.data_as_string().unwrap()),
        None => println!("No message found"),
    }
    Ok(())
}

/// Removes a chunk from a PNG file and saves the result
pub fn remove(file_path: &str, chunk_type: &str) -> Result<(), CommandError> {
    let mut png = read_png(file_path)?;

    let chunk = png.remove_chunk(chunk_type)?;
    save_png(file_path, png)?;

    println!("Removed chunk: {chunk}");
    Ok(())
}

/// Prints all of the chunks in a PNG file
pub fn print_chunks(file_path: &str) -> Result<(), CommandError> {
    let png = read_png(file_path)?;

    println!("{png}");
    Ok(())
}
