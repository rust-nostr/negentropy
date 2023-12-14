// Copyright (c) 2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::ops::Deref;
use std::sync::Arc;

use parking_lot::RwLock;
use uniffi::{Object, Record};

mod bytes;
mod error;
mod storage;

pub use self::bytes::Bytes;
pub use self::error::NegentropyError;
use self::error::Result;
pub use self::storage::NegentropyStorageVector;

#[derive(Record)]
pub struct ReconcileWithIds {
    pub have_ids: Vec<Arc<Bytes>>,
    pub need_ids: Vec<Arc<Bytes>>,
    pub output: Option<Arc<Bytes>>,
}

#[derive(Object)]
pub struct Negentropy {
    inner: RwLock<negentropy::Negentropy<negentropy::NegentropyStorageVector>>,
}

#[uniffi::export]
impl Negentropy {
    /// Create new negentropy instance
    ///
    /// Frame size limit must be `equal to 0` or `greater than 4096`
    #[uniffi::constructor]
    pub fn new(
        storage: Arc<NegentropyStorageVector>,
        frame_size_limit: Option<u64>,
    ) -> Result<Arc<Self>> {
        Ok(Arc::new(Self {
            inner: RwLock::new(negentropy::Negentropy::new(
                storage.as_ref().to_inner(),
                frame_size_limit.unwrap_or_default(),
            )?),
        }))
    }

    /// Initiate reconciliation set
    pub fn initiate(&self) -> Result<Arc<Bytes>> {
        let mut negentropy = self.inner.write();
        Ok(Arc::new(negentropy.initiate()?.into()))
    }

    pub fn is_initiator(&self) -> bool {
        self.inner.read().is_initiator()
    }

    /// Set Initiator: for resuming initiation flow with a new instance
    pub fn set_initiator(&self) {
        let mut negentropy = self.inner.write();
        negentropy.set_initiator();
    }

    pub fn reconcile(&self, query: Arc<Bytes>) -> Result<Arc<Bytes>> {
        let mut negentropy = self.inner.write();
        Ok(Arc::new(
            negentropy.reconcile(query.as_ref().deref())?.into(),
        ))
    }

    pub fn reconcile_with_ids(&self, query: Arc<Bytes>) -> Result<ReconcileWithIds> {
        let mut negentropy = self.inner.write();
        let mut have_ids: Vec<negentropy::Bytes> = Vec::new();
        let mut need_ids: Vec<negentropy::Bytes> = Vec::new();
        let output: Option<negentropy::Bytes> =
            negentropy.reconcile_with_ids(query.as_ref().deref(), &mut have_ids, &mut need_ids)?;
        Ok(ReconcileWithIds {
            have_ids: have_ids.into_iter().map(|id| Arc::new(id.into())).collect(),
            need_ids: need_ids.into_iter().map(|id| Arc::new(id.into())).collect(),
            output: output.map(|o| Arc::new(o.into())),
        })
    }
}

uniffi::setup_scaffolding!("negentropy");
