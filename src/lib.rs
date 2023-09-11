// Copyright (c) 2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Rust implementation of the negentropy set-reconcilliation protocol.

#![warn(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

#[cfg(not(feature = "std"))]
use alloc::collections::BTreeSet as AllocSet;
use alloc::collections::VecDeque;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use core::cmp::Ordering;
use core::fmt;
use core::iter;
use core::ops::BitXorAssign;
#[cfg(feature = "std")]
use std::collections::HashSet as AllocSet;

const MAX_U64: u64 = u64::MAX;
const BUCKETS: usize = 16;
const DOUBLE_BUCKETS: usize = BUCKETS * 2;

/// Error
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// ID too big
    IdTooBig,
    /// Invalid ID size
    InvalidIdSize,
    /// Frame size limit too small
    FrameSizeLimitTooSmall,
    /// Not sealed
    NotSealed,
    /// Already sealed
    AlreadySealed,
    /// Initiator error
    Initiator,
    /// Non-initiator error
    NonInitiator,
    /// Deprecated protocol
    DeprecatedProtocol,
    /// Unexpected mode
    UnexpectedMode(u64),
    /// Parse ends prematurely
    ParseEndsPrematurely,
    /// Prepature end of var int
    PrematureEndOfVarInt,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IdTooBig => write!(f, "ID too big"),
            Self::InvalidIdSize => write!(f, "Invalid ID size"),
            Self::FrameSizeLimitTooSmall => write!(f, "Frame size limit too small"),
            Self::NotSealed => write!(f, "Not sealed"),
            Self::AlreadySealed => write!(f, "Already sealed"),
            Self::Initiator => write!(f, "initiator not asking for have/need IDs"),
            Self::NonInitiator => write!(f, "non-initiator asking for have/need IDs"),
            Self::DeprecatedProtocol => write!(f, "Other side is speaking old negentropy protocol"),
            Self::UnexpectedMode(m) => write!(f, "Unexpected mode: {}", m),
            Self::ParseEndsPrematurely => write!(f, "parse ends prematurely"),
            Self::PrematureEndOfVarInt => write!(f, "premature end of varint"),
        }
    }
}

#[derive(Debug, Clone, Default)]
struct XorElem {
    timestamp: u64,
    id_size: u8,
    id: Vec<u8>,
}

impl PartialEq for XorElem {
    fn eq(&self, other: &Self) -> bool {
        self.timestamp == other.timestamp && self.get_id() == other.get_id()
    }
}

impl Eq for XorElem {}

impl PartialOrd for XorElem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for XorElem {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.timestamp != other.timestamp {
            self.timestamp.cmp(&other.timestamp)
        } else {
            self.get_id().cmp(other.get_id())
        }
    }
}

impl BitXorAssign for XorElem {
    fn bitxor_assign(&mut self, other: Self) {
        for i in 0..self.id.len() {
            self.id[i] ^= other.id[i];
        }
    }
}

impl XorElem {
    fn new() -> Self {
        Self::default()
    }

    fn with_timestamp_and_id<T>(timestamp: u64, id: T) -> Result<Self, Error>
    where
        T: AsRef<[u8]>,
    {
        let id: &[u8] = id.as_ref();
        let id_len: usize = id.len();

        if id_len > 32 {
            return Err(Error::IdTooBig);
        }

        let mut xor_elem = Self::new();
        xor_elem.timestamp = timestamp;
        xor_elem.id_size = id_len as u8;

        if id_len > 0 {
            xor_elem.id.extend(id);
        }

        Ok(xor_elem)
    }

    fn get_id(&self) -> &[u8] {
        self.id.get(..self.id_size as usize).unwrap_or_default()
    }

    fn get_id_subsize(&self, sub_size: u64) -> &[u8] {
        self.id.get(..sub_size as usize).unwrap_or_default()
    }
}

#[derive(Debug, Clone)]
struct BoundOutput {
    start: XorElem,
    end: XorElem,
    payload: Vec<u8>,
}

/// Negentropy
#[derive(Debug, Clone)]
pub struct Negentropy {
    id_size: u64,
    frame_size_limit: Option<u64>,
    items: Vec<XorElem>,
    sealed: bool,
    is_initiator: bool,
    continuation_needed: bool,
    pending_outputs: VecDeque<BoundOutput>,
}

impl Negentropy {
    /// Create new [`Negentropy`] instance
    pub fn new(id_size: u8, frame_size_limit: Option<u64>) -> Result<Self, Error> {
        if !(8..=32).contains(&id_size) {
            return Err(Error::InvalidIdSize);
        }

        if let Some(frame_size_limit) = frame_size_limit {
            if frame_size_limit > 0 && frame_size_limit < 4096 {
                return Err(Error::FrameSizeLimitTooSmall);
            }
        }

        Ok(Self {
            id_size: id_size as u64,
            frame_size_limit,
            items: Vec::new(),
            sealed: false,
            is_initiator: false,
            continuation_needed: false,
            pending_outputs: VecDeque::new(),
        })
    }

    /// Check if current instance it's an initiator
    pub fn is_initiator(&self) -> bool {
        self.is_initiator
    }

    /// Check if need to continue
    pub fn continuation_needed(&self) -> bool {
        self.continuation_needed
    }

    /// Add item
    pub fn add_item<T>(&mut self, created_at: u64, id: T) -> Result<(), Error>
    where
        T: AsRef<[u8]>,
    {
        if self.sealed {
            return Err(Error::AlreadySealed);
        }

        let id: &[u8] = id.as_ref();

        if id.len() != self.id_size as usize {
            return Err(Error::InvalidIdSize);
        }

        let elem = XorElem::with_timestamp_and_id(created_at, id)?;

        self.items.push(elem);
        Ok(())
    }

    /// Seal
    pub fn seal(&mut self) -> Result<(), Error> {
        if self.sealed {
            return Err(Error::AlreadySealed);
        }

        self.items.sort();
        self.sealed = true;
        Ok(())
    }

    /// Initiate reconcilliation set
    pub fn initiate(&mut self) -> Result<String, Error> {
        if !self.sealed {
            return Err(Error::NotSealed);
        }

        self.is_initiator = true;
        let items: Vec<XorElem> = self.items.clone();
        let mut outputs: VecDeque<BoundOutput> = self.pending_outputs.clone();

        self.split_range(
            &items,
            XorElem::new(),
            XorElem::with_timestamp_and_id(MAX_U64, [])?,
            &mut outputs,
        )?;

        self.pending_outputs = outputs;

        self.build_output()
    }

    /// Reconcilie
    pub fn reconcile<T>(&mut self, query: T) -> Result<String, Error>
    where
        T: AsRef<[u8]>,
    {
        if self.is_initiator {
            return Err(Error::Initiator);
        }

        self.reconcile_aux(query, &mut Vec::new(), &mut Vec::new())?;
        self.build_output()
    }

    /// Reconcilie
    pub fn reconcile_with_ids<T>(
        &mut self,
        query: T,
        have_ids: &mut Vec<String>,
        need_ids: &mut Vec<String>,
    ) -> Result<String, Error>
    where
        T: AsRef<[u8]>,
    {
        if !self.is_initiator {
            return Err(Error::NonInitiator);
        }

        self.reconcile_aux(query, have_ids, need_ids)?;
        self.build_output()
    }

    fn reconcile_aux<T>(
        &mut self,
        query: T,
        have_ids: &mut Vec<String>,
        need_ids: &mut Vec<String>,
    ) -> Result<(), Error>
    where
        T: AsRef<[u8]>,
    {
        if !self.sealed {
            return Err(Error::NotSealed);
        }

        self.continuation_needed = false;

        let mut prev_bound: XorElem = XorElem::new();
        let mut prev_index: usize = 0;
        let mut last_timestamp_in: u64 = 0;
        let mut outputs: VecDeque<BoundOutput> = VecDeque::new();
        let mut query: Vec<u8> = query.as_ref().to_vec();

        while !query.is_empty() {
            let curr_bound: XorElem = self.decode_bound(&mut query, &mut last_timestamp_in)?;
            let mode: u64 = self.decode_var_int(&mut query)?;

            let lower = prev_index;
            let mut upper = self.items.len();
            for i in prev_index..self.items.len() {
                if self.items[i] >= curr_bound {
                    upper = i;
                    break;
                }
            }

            match mode {
                0 => {
                    // Skip
                }
                1 => {
                    // Fingerprint
                    let their_xor_set: XorElem = XorElem::with_timestamp_and_id(
                        0,
                        &self.get_bytes(&mut query, self.id_size)?,
                    )?;

                    let mut our_xor_set: XorElem = XorElem::new();
                    for i in lower..upper {
                        our_xor_set ^= self.items[i].clone();
                    }

                    if their_xor_set.get_id() != our_xor_set.get_id_subsize(self.id_size) {
                        let items = self.items.clone();
                        self.split_range(
                            &items[lower..upper],
                            prev_bound,
                            curr_bound.clone(),
                            &mut outputs,
                        )?;
                    }
                }
                2 => {
                    // IdList
                    let num_ids: u64 = self.decode_var_int(&mut query)?;
                    let mut their_elems: AllocSet<Vec<u8>> = AllocSet::new();

                    for _ in 0..num_ids {
                        let e: Vec<u8> = self.get_bytes(&mut query, self.id_size)?;
                        their_elems.insert(e);
                    }

                    for i in lower..upper {
                        let k = self.items[i].get_id();
                        if !their_elems.contains(k) {
                            if self.is_initiator {
                                have_ids.push(String::from_utf8_lossy(k).to_string());
                            }
                        } else {
                            their_elems.remove(k);
                        }
                    }

                    if self.is_initiator {
                        for k in their_elems.into_iter() {
                            need_ids.push(String::from_utf8_lossy(&k).to_string());
                        }
                    } else {
                        let mut response_have_ids: Vec<&[u8]> = Vec::new();
                        let mut it = lower;
                        let mut did_split = false;
                        let mut split_bound = XorElem::new();

                        while it < upper {
                            let k: &[u8] = self.items[it].get_id();
                            response_have_ids.push(k);
                            if response_have_ids.len() >= 100 {
                                self.flush_id_list_output(
                                    &mut outputs,
                                    upper,
                                    prev_bound.clone(),
                                    &mut did_split,
                                    &mut it,
                                    &mut split_bound,
                                    &curr_bound,
                                    &mut response_have_ids,
                                )?;
                            }
                            it += 1;
                        }
                        self.flush_id_list_output(
                            &mut outputs,
                            upper,
                            prev_bound,
                            &mut did_split,
                            &mut it,
                            &mut split_bound,
                            &curr_bound,
                            &mut response_have_ids,
                        )?;
                    }
                }
                3 => {
                    // Deprecated
                    return Err(Error::DeprecatedProtocol);
                }
                4 => {
                    // Continuation
                    self.continuation_needed = true;
                }
                m => {
                    return Err(Error::UnexpectedMode(m));
                }
            }

            prev_index = upper;
            prev_bound = curr_bound.clone();
        }

        while let Some(output) = outputs.pop_front() {
            self.pending_outputs.push_front(output);
        }

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn flush_id_list_output(
        &self,
        outputs: &mut VecDeque<BoundOutput>,
        upper: usize,
        prev_bound: XorElem,
        did_split: &mut bool,
        it: &mut usize,
        split_bound: &mut XorElem,
        curr_bound: &XorElem,
        response_have_ids: &mut Vec<&[u8]>,
    ) -> Result<(), Error> {
        let mut payload: Vec<u8> = self.encode_var_int(2); // mode = IdList
        payload.extend(self.encode_var_int(response_have_ids.len() as u64));

        for id in response_have_ids.iter() {
            payload.extend_from_slice(id);
        }

        let next_split_bound: XorElem = if *it + 1 >= upper {
            curr_bound.clone()
        } else {
            self.get_minimal_bound(&self.items[*it], &self.items[*it + 1])?
        };

        outputs.push_back(BoundOutput {
            start: if *did_split {
                split_bound.clone()
            } else {
                prev_bound
            },
            end: next_split_bound.clone(),
            payload,
        });

        *split_bound = next_split_bound;
        *did_split = true;

        response_have_ids.clear();

        Ok(())
    }

    fn split_range(
        &mut self,
        items: &[XorElem],
        lower_bound: XorElem,
        upper_bound: XorElem,
        outputs: &mut VecDeque<BoundOutput>,
    ) -> Result<(), Error> {
        let num_elems: usize = items.len();

        if num_elems < DOUBLE_BUCKETS {
            let mut payload: Vec<u8> = self.encode_var_int(2); // mode = IdList
            payload.extend(self.encode_var_int(num_elems as u64));

            for elem in items.iter() {
                payload.extend_from_slice(elem.get_id_subsize(self.id_size));
            }

            outputs.push_back(BoundOutput {
                start: lower_bound,
                end: upper_bound,
                payload,
            });
        } else {
            let items_per_bucket: usize = num_elems / BUCKETS;
            let buckets_with_extra: usize = num_elems % BUCKETS;
            let lower: XorElem = items.first().cloned().unwrap_or_default();
            let mut prev_bound: XorElem = lower.clone();
            let curr = items.iter().cloned().peekable();

            for i in 0..BUCKETS {
                let mut our_xor_set = XorElem::new();

                let bucket_end = curr.clone().take(items_per_bucket);
                if i < buckets_with_extra {
                    for elem in bucket_end.chain(iter::once(lower.clone())) {
                        our_xor_set ^= elem;
                    }
                } else {
                    for elem in bucket_end {
                        our_xor_set ^= elem;
                    }
                };

                let mut payload: Vec<u8> = self.encode_var_int(1); // mode = Fingerprint
                payload.extend(our_xor_set.get_id_subsize(self.id_size));

                let next_bound = if i == 0 {
                    lower_bound.clone()
                } else {
                    self.get_minimal_bound(&prev_bound, &lower)?
                };

                outputs.push_back(BoundOutput {
                    start: if i == 0 {
                        lower_bound.clone()
                    } else {
                        prev_bound
                    },
                    end: upper_bound.clone(),
                    payload,
                });

                prev_bound = next_bound;
            }

            if let Some(output) = outputs.back_mut() {
                output.end = upper_bound;
            }
        }

        Ok(())
    }

    fn build_output(&mut self) -> Result<String, Error> {
        let mut output: Vec<u8> = Vec::new();
        let mut curr_bound: XorElem = XorElem::new();
        let mut last_timestamp_out: u64 = 0;

        self.pending_outputs
            .make_contiguous()
            .sort_by(|a, b| a.start.cmp(&b.start));

        while let Some(p) = self.pending_outputs.front() {
            let mut o: Vec<u8> = Vec::new();

            if p.start < curr_bound {
                break;
            }

            if curr_bound != p.start {
                o.extend(self.encode_bound(&p.start, &mut last_timestamp_out));
                o.extend(self.encode_var_int(0)); // mode = Skip
            }

            o.extend(self.encode_bound(&p.end, &mut last_timestamp_out));
            o.extend(&p.payload);

            if let Some(frame_size_limit) = self.frame_size_limit {
                if frame_size_limit > 0 && output.len() + o.len() > (frame_size_limit - 5) as usize
                {
                    // 5 leaves room for Continuation
                    break;
                }
            }

            output.extend(o);
            curr_bound = p.end.clone();
            self.pending_outputs.pop_front();
        }

        if (!self.is_initiator && !self.pending_outputs.is_empty())
            || (self.is_initiator && output.is_empty() && self.continuation_needed)
        {
            output.extend(&self.encode_bound(
                &XorElem::with_timestamp_and_id(MAX_U64, [])?,
                &mut last_timestamp_out,
            ));
            output.extend(&self.encode_var_int(4)); // mode = Continue
        }

        Ok(String::from_utf8_lossy(&output).to_string())
    }

    fn get_bytes(&self, encoded: &mut Vec<u8>, n: u64) -> Result<Vec<u8>, Error> {
        let n = n as usize;
        if encoded.len() < n {
            return Err(Error::ParseEndsPrematurely);
        }
        Ok(encoded.drain(..n).collect())
    }

    fn decode_var_int(&self, encoded: &mut Vec<u8>) -> Result<u64, Error> {
        let mut res = 0u64;

        loop {
            if encoded.is_empty() {
                return Err(Error::PrematureEndOfVarInt);
            }
            let byte = encoded.remove(0) as u64;
            res = (res << 7) | (byte & 0b0111_1111);
            if (byte & 0b1000_0000) == 0 {
                break;
            }
        }

        Ok(res)
    }

    fn decode_timestamp_in(
        &self,
        encoded: &mut Vec<u8>,
        last_timestamp_in: &mut u64,
    ) -> Result<u64, Error> {
        let timestamp: u64 = self.decode_var_int(encoded)?;
        let mut timestamp = if timestamp == 0 {
            MAX_U64
        } else {
            timestamp - 1
        };
        timestamp += *last_timestamp_in;
        if timestamp < *last_timestamp_in {
            timestamp = MAX_U64;
        }
        *last_timestamp_in = timestamp;
        Ok(timestamp)
    }

    fn decode_bound(
        &self,
        encoded: &mut Vec<u8>,
        last_timestamp_in: &mut u64,
    ) -> Result<XorElem, Error> {
        let timestamp = self.decode_timestamp_in(encoded, last_timestamp_in)?;
        let len = self.decode_var_int(encoded)?;
        let id = self.get_bytes(encoded, len)?;
        XorElem::with_timestamp_and_id(timestamp, id)
    }

    fn encode_var_int(&self, mut n: u64) -> Vec<u8> {
        if n == 0 {
            return vec![0];
        }

        let mut o: Vec<u8> = Vec::new();

        while n > 0 {
            o.push((n & 0x7F) as u8);
            n >>= 7;
        }

        o.into_iter().rev().collect()
    }

    fn encode_timestamp_out(&self, timestamp: u64, last_timestamp_out: &mut u64) -> Vec<u8> {
        if timestamp == MAX_U64 {
            *last_timestamp_out = MAX_U64;
            return self.encode_var_int(0);
        }

        let temp: u64 = timestamp;
        let timestamp: u64 = timestamp.saturating_sub(*last_timestamp_out);
        *last_timestamp_out = temp;
        self.encode_var_int(timestamp.saturating_add(1))
    }

    fn encode_bound(&self, bound: &XorElem, last_timestamp_out: &mut u64) -> Vec<u8> {
        let mut output: Vec<u8> = Vec::new();
        output.extend(self.encode_timestamp_out(bound.timestamp, last_timestamp_out));
        output.extend(&self.encode_var_int(bound.id_size as u64));
        output.extend(bound.get_id_subsize(self.id_size));
        output
    }

    fn get_minimal_bound(&self, prev: &XorElem, curr: &XorElem) -> Result<XorElem, Error> {
        if curr.timestamp != prev.timestamp {
            XorElem::with_timestamp_and_id(curr.timestamp, [])
        } else {
            let mut shared_prefix_bytes: usize = 0;
            let curr_key: &[u8] = curr.get_id();
            let prev_key: &[u8] = prev.get_id();

            for i in 0..self.id_size {
                if curr_key[i as usize] != prev_key[i as usize] {
                    break;
                }
                shared_prefix_bytes += 1;
            }

            XorElem::with_timestamp_and_id(curr.timestamp, &curr_key[..shared_prefix_bytes + 1])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reconciliation_set() {
        // Client
        let mut client = Negentropy::new(16, None).unwrap();
        client.add_item(0, "aaaaaaaaaaaaaaaa").unwrap();
        client.add_item(1, "bbbbbbbbbbbbbbbb").unwrap();
        client.seal().unwrap();
        let init_output = client.initiate().unwrap();

        // Relay
        let mut relay = Negentropy::new(16, None).unwrap();
        relay.add_item(0, "aaaaaaaaaaaaaaaa").unwrap();
        relay.add_item(2, "cccccccccccccccc").unwrap();
        relay.add_item(3, "1111111111111111").unwrap();
        relay.add_item(5, "2222222222222222").unwrap();
        relay.add_item(10, "3333333333333333").unwrap();
        relay.seal().unwrap();
        let reconcile_output = relay.reconcile(&init_output).unwrap();

        // Client
        let mut have_ids = Vec::new();
        let mut need_ids = Vec::new();
        let reconcile_output_with_ids = client
            .reconcile_with_ids(&reconcile_output, &mut have_ids, &mut need_ids)
            .unwrap();

        // Check reconcile with IDs output
        assert!(reconcile_output_with_ids.is_empty());

        // Check have IDs
        assert!(have_ids.contains(&String::from("bbbbbbbbbbbbbbbb")));

        // Check need IDs
        #[cfg(feature = "std")]
        need_ids.sort();
        assert_eq!(
            need_ids,
            vec![
                String::from("1111111111111111"),
                String::from("2222222222222222"),
                String::from("3333333333333333"),
                String::from("cccccccccccccccc"),
            ]
        )
    }

    #[test]
    fn test_invalid_id_size() {
        assert_eq!(Negentropy::new(33, None).unwrap_err(), Error::InvalidIdSize);

        let mut client = Negentropy::new(16, None).unwrap();
        assert_eq!(
            client.add_item(0, "item").unwrap_err(),
            Error::InvalidIdSize
        );
    }
}
