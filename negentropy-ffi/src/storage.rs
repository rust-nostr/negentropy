// Copyright (c) 2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::sync::Mutex;

use negentropy::NegentropyStorageBase;
use uniffi::Object;

use crate::error::Result;
use crate::id::Id;

#[derive(Object)]
pub struct NegentropyStorageVector {
    inner: Mutex<negentropy::NegentropyStorageVector>,
}

#[uniffi::export]
impl NegentropyStorageVector {
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(negentropy::NegentropyStorageVector::new()),
        }
    }

    /// Insert item
    pub fn insert(&self, created_at: u64, id: &Id) -> Result<()> {
        let mut storage = self.inner.lock()?;
        Ok(storage.insert(created_at, **id)?)
    }

    /// Seal
    pub fn seal(&self) -> Result<()> {
        let mut storage = self.inner.lock()?;
        Ok(storage.seal()?)
    }

    /// Unseal
    pub fn unseal(&self) -> Result<()> {
        let mut storage = self.inner.lock()?;
        Ok(storage.unseal()?)
    }

    fn size(&self) -> Result<u64> {
        let storage = self.inner.lock()?;
        Ok(storage.size()? as u64)
    }
}

impl NegentropyStorageVector {
    pub(crate) fn to_inner(&self) -> Result<negentropy::NegentropyStorageVector> {
        let storage = self.inner.lock()?;
        Ok(storage.to_owned())
    }
}
