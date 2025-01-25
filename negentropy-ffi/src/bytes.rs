// Copyright (c) 2023 Yuki Kishimoto
// Distributed under the MIT software license

use core::ops::Deref;
use std::sync::Arc;

use uniffi::Object;

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

    pub fn as_bytes(&self) -> Vec<u8> {
        self.inner.as_bytes().to_vec()
    }
}
