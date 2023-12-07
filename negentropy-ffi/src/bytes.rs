// Copyright (c) 2023 Yuki Kishimoto
// Distributed under the MIT software license

use core::ops::Deref;
use std::sync::Arc;

use uniffi::Object;

use crate::error::Result;

#[derive(Object)]
pub struct Bytes {
    inner: negentropy::Bytes,
}

impl Deref for Bytes {
    type Target = negentropy::Bytes;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<negentropy::Bytes> for Bytes {
    fn from(inner: negentropy::Bytes) -> Self {
        Self { inner }
    }
}

#[uniffi::export]
impl Bytes {
    #[uniffi::constructor]
    pub fn new(bytes: Vec<u8>) -> Arc<Self> {
        Arc::new(Self {
            inner: negentropy::Bytes::new(bytes),
        })
    }

    #[uniffi::constructor]
    pub fn from_hex(data: String) -> Result<Arc<Self>> {
        Ok(Arc::new(Self {
            inner: negentropy::Bytes::from_hex(data)?,
        }))
    }

    pub fn as_hex(&self) -> String {
        self.inner.as_hex()
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        self.inner.as_bytes().to_vec()
    }
}
