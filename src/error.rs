use crate::bspfile::LumpType;
use crate::data::*;
use std::num::{ParseFloatError, ParseIntError};
use thiserror::Error;
use zip::result::ZipError;

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum BspError {
    #[error("unexpected magic numbers or version")]
    UnexpectedHeader(Header),
    #[error("bsp lump is out of bounds of the bsp file")]
    LumpOutOfBounds(LumpEntry),
    #[error("bsp game lump is out of bounds of the bsp file")]
    GameLumpOutOfBounds(GameLump),
    #[error("compressed game lump is malformed")]
    MalformedCompressedGameLump,
    #[error("Invalid lump size, lump size {lump_size} is not a multiple of the element size {element_size}")]
    InvalidLumpSize {
        lump: LumpType,
        element_size: usize,
        lump_size: usize,
    },
    #[error("unexpected length of uncompressed lump, got {got} but expected {expected}")]
    UnexpectedUncompressedLumpSize { got: u32, expected: u32 },
    #[error("unexpected length of compressed lump, got {got} but expected {expected}")]
    UnexpectedCompressedLumpSize { got: u32, expected: u32 },
    #[error("error while decompressing lump")]
    LumpDecompressError(lzma_rs::error::Error),
    #[error("io error while reading data: {0}")]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    String(#[from] StringError),
    #[error("Malformed field found while parsing: {0:#}")]
    MalformedData(binrw::Error),
    #[error("bsp file is well-formed but contains invalid data")]
    Validation(#[from] ValidationError),
    #[error(transparent)]
    LumpVersion(UnsupportedLumpVersion),
    #[error(transparent)]
    Zip(#[from] ZipError),
}

impl From<binrw::Error> for BspError {
    fn from(e: binrw::Error) -> Self {
        use binrw::Error;

        // only a few error types should be generated by our code
        match e {
            Error::Io(e) => BspError::IO(e),
            Error::Custom { err, .. } => {
                if err.is::<StringError>() {
                    BspError::String(*err.downcast::<StringError>().unwrap())
                } else if err.is::<UnsupportedLumpVersion>() {
                    BspError::LumpVersion(*err.downcast::<UnsupportedLumpVersion>().unwrap())
                } else {
                    panic!("unexpected custom error")
                }
            }
            e => BspError::MalformedData(e),
        }
    }
}

impl From<lzma_rs::error::Error> for BspError {
    fn from(e: lzma_rs::error::Error) -> Self {
        use lzma_rs::error::Error;

        match e {
            Error::IoError(e) => BspError::IO(e),
            e => BspError::LumpDecompressError(e),
        }
    }
}

#[derive(Debug, Error)]
pub enum StringError {
    #[error(transparent)]
    NonUTF8(#[from] std::str::Utf8Error),
    #[error("String is not null-terminated")]
    NotNullTerminated,
}

#[derive(Debug, Error)]
#[error("Unsupported lump version {version} for {lump_type} lump")]
pub struct UnsupportedLumpVersion {
    pub lump_type: &'static str,
    pub version: u16,
}

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error(
    "A {source_} indexes into {target} but the index {index} is out of range of the size {size}"
    )]
    ReferenceOutOfRange {
        source_: &'static str,
        target: &'static str,
        index: i64,
        size: usize,
    },
    #[error("bsp contains no root node")]
    NoRootNode,
    #[error("displacement face with {0} edges")]
    NonSquareDisplacement(i16),
    #[error("No static prop lump found")]
    NoStaticPropLump,
    #[error(transparent)]
    Neighbour(InvalidNeighbourError),
}

#[derive(Debug, Error)]
pub enum InvalidNeighbourError {
    #[error("Invalid neighbour span")]
    InvalidNeighbourIndex,
    #[error("Invalid neighbour span")]
    InvalidNeighbourSpan(u8),
    #[error("Invalid neighbour orientation")]
    InvalidNeighbourOrientation(u8),
}

#[derive(Debug, Error)]
pub enum EntityParseError {
    #[error("no such property: {0}")]
    NoSuchProperty(&'static str),
    #[error("wrong number of elements")]
    ElementCount,
    #[error(transparent)]
    Float(#[from] ParseFloatError),
    #[error(transparent)]
    Int(#[from] ParseIntError),
}
