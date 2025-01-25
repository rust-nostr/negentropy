// Copyright (c) 2023 Doug Hoyte
// Copyright (c) 2023 Yuki Kishimoto
// Distributed under the MIT software license

use alloc::vec::Vec;
use core::ops::Deref;

/// Bytes
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Bytes(Vec<u8>);

impl Deref for Bytes {
    type Target = Vec<u8>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Bytes {
    /// Construct from bytes
    #[inline]
    pub fn new<T>(bytes: T) -> Self
    where
        T: AsRef<[u8]>,
    {
        Self::from(bytes.as_ref())
    }

    /// Construct from slice
    #[inline]
    pub fn from_slice(slice: &[u8]) -> Self {
        Self::from(slice)
    }

    /// Return the inner value
    #[inline]
    pub fn to_bytes(self) -> Vec<u8> {
        self.0
    }

    /// Return reference to the inner value
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl From<Vec<u8>> for Bytes {
    fn from(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }
}

impl From<&[u8]> for Bytes {
    fn from(bytes: &[u8]) -> Self {
        Self(bytes.to_vec())
    }
}

impl AsRef<[u8]> for Bytes {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}
