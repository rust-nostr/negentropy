// Copyright (c) 2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::ops::Deref;
use std::sync::Arc;

use parking_lot::RwLock;

mod bytes;
mod error;

pub use self::bytes::Bytes;
pub use self::error::NegentropyError;
use self::error::Result;

pub struct ReconcileWithIds {
    pub have_ids: Vec<Arc<Bytes>>,
    pub need_ids: Vec<Arc<Bytes>>,
    pub output: Option<Arc<Bytes>>,
}

pub struct Negentropy {
    inner: RwLock<negentropy::Negentropy>,
}

impl Negentropy {
    pub fn new(id_size: u8, frame_size_limit: Option<u64>) -> Result<Self> {
        Ok(Self {
            inner: RwLock::new(negentropy::Negentropy::new(
                id_size as usize,
                frame_size_limit,
            )?),
        })
    }

    pub fn id_size(&self) -> u64 {
        self.inner.read().id_size() as u64
    }

    /// Check if current instance it's an initiator
    pub fn is_initiator(&self) -> bool {
        self.inner.read().is_initiator()
    }

    /// Check if sealed
    pub fn is_sealed(&self) -> bool {
        self.inner.read().is_sealed()
    }

    /// Check if need to continue
    pub fn continuation_needed(&self) -> bool {
        self.inner.read().continuation_needed()
    }

    pub fn add_item(&self, created_at: u64, id: Arc<Bytes>) -> Result<()> {
        let mut negentropy = self.inner.write();
        Ok(negentropy.add_item(created_at, id.as_ref().deref().clone())?)
    }

    pub fn seal(&self) -> Result<()> {
        let mut negentropy = self.inner.write();
        Ok(negentropy.seal()?)
    }

    /// Initiate reconciliation set
    pub fn initiate(&self) -> Result<Arc<Bytes>> {
        let mut negentropy = self.inner.write();
        Ok(Arc::new(negentropy.initiate()?.into()))
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

// UDL
uniffi::include_scaffolding!("negentropy");
