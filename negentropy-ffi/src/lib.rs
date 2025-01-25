// Copyright (c) 2023 Yuki Kishimoto
// Distributed under the MIT software license

#![allow(clippy::new_without_default)]

use std::sync::{Arc, Mutex};

use uniffi::{Object, Record};

mod error;
mod id;
mod storage;

pub use self::error::NegentropyError;
use self::error::Result;
pub use self::storage::NegentropyStorageVector;
use crate::id::Id;

#[derive(Record)]
pub struct ReconcileWithIds {
    pub have_ids: Vec<Arc<Id>>,
    pub need_ids: Vec<Arc<Id>>,
    pub output: Option<Vec<u8>>,
}

#[derive(Object)]
pub struct Negentropy {
    inner: Mutex<negentropy::Negentropy<'static, negentropy::NegentropyStorageVector>>,
}

#[uniffi::export]
impl Negentropy {
    /// Create new negentropy instance
    ///
    /// Frame size limit must be `equal to 0` or `greater than 4096`
    #[uniffi::constructor]
    pub fn new(storage: &NegentropyStorageVector, frame_size_limit: Option<u64>) -> Result<Self> {
        Ok(Self {
            inner: Mutex::new(negentropy::Negentropy::owned(
                storage.to_inner()?,
                frame_size_limit.unwrap_or_default(),
            )?),
        })
    }

    /// Initiate reconciliation set
    pub fn initiate(&self) -> Result<Vec<u8>> {
        let mut negentropy = self.inner.lock()?;
        Ok(negentropy.initiate()?)
    }

    pub fn is_initiator(&self) -> Result<bool> {
        let negentropy = self.inner.lock()?;
        Ok(negentropy.is_initiator())
    }

    /// Set initiator: for resuming initiation flow with a new instance
    pub fn set_initiator(&self) -> Result<()> {
        let mut negentropy = self.inner.lock()?;
        negentropy.set_initiator();
        Ok(())
    }

    /// Reconcile (server method)
    pub fn reconcile(&self, query: &[u8]) -> Result<Vec<u8>> {
        let mut negentropy = self.inner.lock()?;
        Ok(negentropy.reconcile(query)?)
    }

    /// Reconcile (client method)
    pub fn reconcile_with_ids(&self, query: &[u8]) -> Result<ReconcileWithIds> {
        let mut negentropy = self.inner.lock()?;
        let mut have_ids = Vec::new();
        let mut need_ids = Vec::new();
        let output: Option<Vec<u8>> =
            negentropy.reconcile_with_ids(query, &mut have_ids, &mut need_ids)?;
        Ok(ReconcileWithIds {
            have_ids: have_ids.into_iter().map(|id| Arc::new(id.into())).collect(),
            need_ids: need_ids.into_iter().map(|id| Arc::new(id.into())).collect(),
            output,
        })
    }
}

uniffi::setup_scaffolding!("negentropy");
