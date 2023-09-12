// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
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
            Self::InvalidHexCharacter { c, index } => {
                write!(f, "Invalid character {} at position {}", c, index)
            }
            Self::OddLength => write!(f, "Odd number of digits"),
        }
    }
}

pub fn encode<T>(data: T) -> String
where
    T: AsRef<[u8]>,
{
    let bytes: &[u8] = data.as_ref();
    let mut hex: String = String::with_capacity(2 * bytes.len());
    bytes
        .iter()
        .for_each(|b| hex.push_str(format!("{:02X}", b).as_str()));
    hex.to_lowercase()
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
    if hex.len() % 2 != 0 {
        return Err(Error::OddLength);
    }
    hex.chunks(2)
        .enumerate()
        .map(|(i, pair)| Ok(val(pair[0], 2 * i)? << 4 | val(pair[1], 2 * i + 1)?))
        .collect()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_encode() {
        assert_eq!(encode("foobar"), "666f6f626172");
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
