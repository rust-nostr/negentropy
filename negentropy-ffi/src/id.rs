// Copyright (c) 2023 Yuki Kishimoto
// Distributed under the MIT software license

use core::ops::Deref;

use uniffi::Object;

use crate::error::Result;

#[derive(Object)]
pub struct Id {
    inner: negentropy::Id,
}

impl Deref for Id {
    type Target = negentropy::Id;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<negentropy::Id> for Id {
    fn from(inner: negentropy::Id) -> Self {
        Self { inner }
    }
}

#[uniffi::export]
impl Id {
    #[uniffi::constructor]
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self> {
        Ok(Self {
            inner: negentropy::Id::from_slice(&bytes)?,
        })
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        self.inner.as_bytes().to_vec()
    }
}
