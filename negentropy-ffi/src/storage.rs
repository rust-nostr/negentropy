// Copyright (c) 2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::ops::Deref;
use std::sync::Arc;

use negentropy::NegentropyStorageBase;
use parking_lot::RwLock;
use uniffi::Object;

use crate::error::Result;
use crate::Bytes;

#[derive(Object)]
pub struct NegentropyStorageVector {
    inner: Arc<RwLock<negentropy::NegentropyStorageVector>>,
}

#[uniffi::export]
impl NegentropyStorageVector {
    #[uniffi::constructor]
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            inner: Arc::new(RwLock::new(negentropy::NegentropyStorageVector::new())),
        })
    }

    /// Insert item
    pub fn insert(&self, created_at: u64, id: Arc<Bytes>) -> Result<()> {
        let mut storage = self.inner.write();
        Ok(storage.insert(created_at, id.as_ref().deref().clone())?)
    }

    /// Seal
    pub fn seal(&self) -> Result<()> {
        let mut storage = self.inner.write();
        Ok(storage.seal()?)
    }

    /// Unseal
    pub fn unseal(&self) -> Result<()> {
        let mut storage = self.inner.write();
        Ok(storage.unseal()?)
    }

    fn size(&self) -> Result<u64> {
        Ok(self.inner.read().size()? as u64)
    }
}

impl NegentropyStorageVector {
    pub(crate) fn to_inner(&self) -> negentropy::NegentropyStorageVector {
        self.inner.read().to_owned()
    }
}
