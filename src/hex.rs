// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// Invalid char
    InvalidChar,
    /// An invalid character was found
    InvalidHexCharacter { c: char, index: usize },
    /// A hex string's length needs to be even, as two digits correspond to
    /// one byte.
    OddLength,
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidChar => write!(f, "Invalid char"),
            Self::InvalidHexCharacter { c, index } => {
                write!(f, "Invalid character {} at position {}", c, index)
            }
            Self::OddLength => write!(f, "Odd number of digits"),
        }
    }
}

pub fn encode<T>(data: T) -> Result<String, Error>
where
    T: AsRef<[u8]>,
{
    let bytes: &[u8] = data.as_ref();
    let mut hex = String::with_capacity(2 * bytes.len());
    for byte in bytes.iter() {
        hex.push(char::from_digit((byte >> 4) as u32, 16).ok_or(Error::InvalidChar)?);
        hex.push(char::from_digit((byte & 0xF) as u32, 16).ok_or(Error::InvalidChar)?);
    }
    Ok(hex.to_lowercase())
}

const fn val(c: u8, idx: usize) -> Result<u8, Error> {
    match c {
        b'A'..=b'F' => Ok(c - b'A' + 10),
        b'a'..=b'f' => Ok(c - b'a' + 10),
        b'0'..=b'9' => Ok(c - b'0'),
        _ => Err(Error::InvalidHexCharacter {
            c: c as char,
            index: idx,
        }),
    }
}

pub fn decode<T>(hex: T) -> Result<Vec<u8>, Error>
where
    T: AsRef<[u8]>,
{
    let hex = hex.as_ref();
    let len = hex.len();

    if len % 2 != 0 {
        return Err(Error::OddLength);
    }

    let mut bytes: Vec<u8> = Vec::with_capacity(len / 2);

    for i in (0..len).step_by(2) {
        let high = val(hex[i], i)?;
        let low = val(hex[i + 1], i + 1)?;
        bytes.push(high << 4 | low);
    }

    Ok(bytes)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_encode() {
        assert_eq!(encode("foobar").unwrap(), "666f6f626172");
    }

    #[test]
    fn test_decode() {
        assert_eq!(
            decode("666f6f626172"),
            Ok(String::from("foobar").into_bytes())
        );
    }

    #[test]
    pub fn test_invalid_length() {
        assert_eq!(decode("1").unwrap_err(), Error::OddLength);
        assert_eq!(decode("666f6f6261721").unwrap_err(), Error::OddLength);
    }

    #[test]
    pub fn test_invalid_char() {
        assert_eq!(
            decode("66ag").unwrap_err(),
            Error::InvalidHexCharacter { c: 'g', index: 3 }
        );
    }
}
