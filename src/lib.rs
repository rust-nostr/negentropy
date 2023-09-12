// Copyright (c) 2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Rust implementation of the negentropy set-reconcilliation protocol.

#![warn(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

use alloc::boxed::Box;
#[cfg(not(feature = "std"))]
use alloc::collections::BTreeSet as AllocSet;
use alloc::string::String;
use alloc::vec::Vec;
use core::cmp::Ordering;
use core::fmt;
use core::iter;
use core::ops::BitXorAssign;
#[cfg(feature = "std")]
use std::collections::HashSet as AllocSet;

mod hex;

const MAX_U64: u64 = u64::MAX;
const BUCKETS: usize = 16;
const DOUBLE_BUCKETS: usize = BUCKETS * 2;

/// Error
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// Hex error
    Hex(hex::Error),
    /// ID too big
    IdTooBig,
    /// Invalid ID size
    InvalidIdSize,
    /// IdSizeNotMatch
    IdSizeNotMatch,
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
            Self::Hex(e) => write!(f, "Hex: {}", e),
            Self::IdTooBig => write!(f, "ID too big"),
            Self::InvalidIdSize => write!(f, "Invalid ID size"),
            Self::IdSizeNotMatch => write!(f, "Current item ID not match the client ID size"),
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

impl From<hex::Error> for Error {
    fn from(e: hex::Error) -> Self {
        Self::Hex(e)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct XorElem {
    timestamp: u64,
    id_size: u8,
    id: [u8; 32],
}

impl XorElem {
    fn new() -> Self {
        Self::default()
    }

    fn with_timestamp(timestamp: u64) -> Self {
        let mut xor_elem = Self::new();
        xor_elem.timestamp = timestamp;
        xor_elem
    }

    fn with_timestamp_and_id<T>(timestamp: u64, id: T) -> Result<Self, Error>
    where
        T: AsRef<[u8]>,
    {
        let id: &[u8] = id.as_ref();
        let len: usize = id.len();

        if len > 32 {
            return Err(Error::IdTooBig);
        }

        let mut xor_elem = Self::new();
        xor_elem.timestamp = timestamp;
        xor_elem.id_size = len as u8;
        xor_elem.id[..len].copy_from_slice(id);

        Ok(xor_elem)
    }

    fn id_size(&self) -> usize {
        self.id_size as usize
    }

    fn get_id(&self) -> &[u8] {
        self.id.get(..self.id_size as usize).unwrap_or_default()
    }

    fn get_id_subsize(&self, sub_size: u64) -> &[u8] {
        self.id.get(..sub_size as usize).unwrap_or_default()
    }
}

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
            self.id.cmp(&other.id)
        }
    }
}

impl BitXorAssign for XorElem {
    fn bitxor_assign(&mut self, other: Self) {
        for i in 0..32 {
            self.id[i] ^= other.id[i];
        }
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
    pending_outputs: Vec<BoundOutput>,
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
            pending_outputs: Vec::new(),
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

        let id: Vec<u8> = hex::decode(id)?;
        if id.len() != self.id_size as usize {
            return Err(Error::IdSizeNotMatch);
        }

        let elem: XorElem = XorElem::with_timestamp_and_id(created_at, id)?;

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

        let mut outputs: Vec<BoundOutput> = Vec::new();

        self.split_range(
            &self.items,
            XorElem::new(),
            XorElem::with_timestamp(MAX_U64),
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
        let query: Vec<u8> = hex::decode(query)?;
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
        let query: Vec<u8> = hex::decode(query)?;
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
        let mut outputs: Vec<BoundOutput> = Vec::new();
        let mut query: &[u8] = query.as_ref();

        while !query.is_empty() {
            let curr_bound: XorElem = self.decode_bound(&mut query, &mut last_timestamp_in)?;
            let mode: u64 = self.decode_var_int(&mut query)?;

            let lower: usize = prev_index;
            let upper: usize = binary_search_upper_bound(&self.items, curr_bound);

            match mode {
                0 => {
                    // Skip
                }
                1 => {
                    // Fingerprint
                    let their_xor_set: XorElem = XorElem::with_timestamp_and_id(
                        0,
                        self.get_bytes(&mut query, self.id_size)?,
                    )?;

                    let mut our_xor_set: XorElem = XorElem::new();
                    for i in lower..upper {
                        our_xor_set ^= self.items[i];
                    }

                    if their_xor_set.get_id() != our_xor_set.get_id_subsize(self.id_size) {
                        self.split_range(
                            &self.items[lower..upper],
                            prev_bound,
                            curr_bound,
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
                                have_ids.push(hex::encode(k));
                            }
                        } else {
                            their_elems.remove(k);
                        }
                    }

                    if self.is_initiator {
                        for k in their_elems.into_iter() {
                            need_ids.push(hex::encode(k));
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
                                    prev_bound,
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
            prev_bound = curr_bound;
        }

        self.pending_outputs.extend(outputs.into_iter().rev());

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn flush_id_list_output(
        &self,
        outputs: &mut Vec<BoundOutput>,
        upper: usize,
        prev_bound: XorElem,
        did_split: &mut bool,
        it: &mut usize,
        split_bound: &mut XorElem,
        curr_bound: &XorElem,
        response_have_ids: &mut Vec<&[u8]>,
    ) -> Result<(), Error> {
        let mut payload: Vec<u8> = Vec::new();
        payload.extend(self.encode_var_int(2)); // mode = IdList
        payload.extend(self.encode_var_int(response_have_ids.len() as u64));

        for id in response_have_ids.iter() {
            payload.extend_from_slice(id);
        }

        let next_split_bound: XorElem = if *it + 1 >= upper {
            *curr_bound
        } else {
            self.get_minimal_bound(&self.items[*it], &self.items[*it + 1])?
        };

        outputs.push(BoundOutput {
            start: if *did_split { *split_bound } else { prev_bound },
            end: next_split_bound,
            payload,
        });

        *split_bound = next_split_bound;
        *did_split = true;

        response_have_ids.clear();

        Ok(())
    }

    fn split_range(
        &self,
        items: &[XorElem],
        lower_bound: XorElem,
        upper_bound: XorElem,
        outputs: &mut Vec<BoundOutput>,
    ) -> Result<(), Error> {
        let num_elems: usize = items.len();

        if num_elems < DOUBLE_BUCKETS {
            let mut payload: Vec<u8> = Vec::new();
            payload.extend(self.encode_var_int(2)); // mode = IdList
            payload.extend(self.encode_var_int(num_elems as u64));

            for elem in items.iter() {
                payload.extend_from_slice(elem.get_id_subsize(self.id_size));
            }

            outputs.push(BoundOutput {
                start: lower_bound,
                end: upper_bound,
                payload,
            });
        } else {
            let items_per_bucket: usize = num_elems / BUCKETS;
            let buckets_with_extra: usize = num_elems % BUCKETS;
            let lower: XorElem = items.first().cloned().unwrap_or_default();
            let mut prev_bound: XorElem = lower;
            let curr = items.iter().cloned().peekable();

            for i in 0..BUCKETS {
                let mut our_xor_set = XorElem::new();

                let bucket_end = curr.clone().take(items_per_bucket);
                if i < buckets_with_extra {
                    for elem in bucket_end.chain(iter::once(lower)) {
                        our_xor_set ^= elem;
                    }
                } else {
                    for elem in bucket_end {
                        our_xor_set ^= elem;
                    }
                };

                let mut payload: Vec<u8> = Vec::new();
                payload.extend(self.encode_var_int(1)); // mode = Fingerprint
                payload.extend(our_xor_set.get_id_subsize(self.id_size));

                let next_bound = if i == 0 {
                    lower_bound
                } else {
                    self.get_minimal_bound(&prev_bound, &lower)?
                };

                outputs.push(BoundOutput {
                    start: if i == 0 { lower_bound } else { prev_bound },
                    end: upper_bound,
                    payload,
                });

                prev_bound = next_bound;
            }

            if let Some(output) = outputs.last_mut() {
                output.end = upper_bound;
            }
        }

        Ok(())
    }

    fn build_output(&mut self) -> Result<String, Error> {
        let mut output: Vec<u8> = Vec::new();
        let mut curr_bound: XorElem = XorElem::new();
        let mut last_timestamp_out: u64 = 0;

        self.pending_outputs.sort_by(|a, b| a.start.cmp(&b.start));

        while let Some(p) = self.pending_outputs.first() {
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
            curr_bound = p.end;
            self.pending_outputs.remove(0);
        }

        if (!self.is_initiator && !self.pending_outputs.is_empty())
            || (self.is_initiator && output.is_empty() && self.continuation_needed)
        {
            output.extend(
                &self.encode_bound(&XorElem::with_timestamp(MAX_U64), &mut last_timestamp_out),
            );
            output.extend(self.encode_var_int(4)); // mode = Continue
        }

        Ok(hex::encode(output))
    }

    fn get_bytes(&self, encoded: &mut &[u8], n: u64) -> Result<Vec<u8>, Error> {
        let n = n as usize;
        if encoded.len() < n {
            return Err(Error::ParseEndsPrematurely);
        }
        let res: Vec<u8> = encoded.get(..n).unwrap_or_default().to_vec();
        *encoded = encoded.get(n..).unwrap_or_default();
        Ok(res)
    }

    fn decode_var_int(&self, encoded: &mut &[u8]) -> Result<u64, Error> {
        let mut res = 0u64;

        for byte in encoded.iter() {
            *encoded = &encoded[1..];
            res = (res << 7) | (*byte as u64 & 0b0111_1111);
            if (byte & 0b1000_0000) == 0 {
                break;
            }
        }

        Ok(res)
    }

    fn decode_timestamp_in(
        &self,
        encoded: &mut &[u8],
        last_timestamp_in: &mut u64,
    ) -> Result<u64, Error> {
        let timestamp: u64 = self.decode_var_int(encoded)?;
        let mut timestamp = if timestamp == 0 {
            MAX_U64
        } else {
            timestamp - 1
        };
        timestamp = timestamp.saturating_add(*last_timestamp_in);
        *last_timestamp_in = timestamp;
        Ok(timestamp)
    }

    fn decode_bound(
        &self,
        encoded: &mut &[u8],
        last_timestamp_in: &mut u64,
    ) -> Result<XorElem, Error> {
        let timestamp = self.decode_timestamp_in(encoded, last_timestamp_in)?;
        let len = self.decode_var_int(encoded)?;
        let id = self.get_bytes(encoded, len)?;
        XorElem::with_timestamp_and_id(timestamp, id)
    }

    fn encode_var_int(&self, mut n: u64) -> Box<dyn Iterator<Item = u8> + '_> {
        if n == 0 {
            return Box::new(iter::once(0));
        }

        let mut o: Vec<u8> = Vec::new();

        while n > 0 {
            o.push((n & 0x7F) as u8);
            n >>= 7;
        }

        Box::new(o.into_iter().rev())
    }

    fn encode_timestamp_out(
        &self,
        timestamp: u64,
        last_timestamp_out: &mut u64,
    ) -> Box<dyn Iterator<Item = u8> + '_> {
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
        output.extend(self.encode_var_int(bound.id_size() as u64));
        output.extend(bound.get_id());
        output
    }

    fn get_minimal_bound(&self, prev: &XorElem, curr: &XorElem) -> Result<XorElem, Error> {
        if curr.timestamp != prev.timestamp {
            Ok(XorElem::with_timestamp(curr.timestamp))
        } else {
            let mut shared_prefix_bytes: usize = 0;
            for i in 0..prev.id_size().min(curr.id_size()) {
                if curr.id[i] != prev.id[i] {
                    break;
                }
                shared_prefix_bytes += 1;
            }
            XorElem::with_timestamp_and_id(curr.timestamp, &curr.id[..shared_prefix_bytes + 1])
        }
    }
}

fn binary_search_upper_bound<T>(items: &[T], curr_bound: T) -> usize
where
    T: Ord,
{
    let mut low = 0;
    let mut high = items.len();

    while low < high {
        let mid = low + (high - low) / 2;
        if items[mid] < curr_bound {
            low = mid + 1;
        } else {
            high = mid;
        }
    }

    low
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::*;

    #[test]
    fn test_reconciliation_set() {
        // Client
        let mut client = Negentropy::new(16, None).unwrap();
        client
            .add_item(0, "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
            .unwrap();
        client
            .add_item(1, "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb")
            .unwrap();
        client.seal().unwrap();
        let init_output = client.initiate().unwrap();

        // Relay
        let mut relay = Negentropy::new(16, None).unwrap();
        relay
            .add_item(0, "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
            .unwrap();
        relay
            .add_item(2, "cccccccccccccccccccccccccccccccc")
            .unwrap();
        relay
            .add_item(3, "11111111111111111111111111111111")
            .unwrap();
        relay
            .add_item(5, "22222222222222222222222222222222")
            .unwrap();
        relay
            .add_item(10, "33333333333333333333333333333333")
            .unwrap();
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
        assert!(have_ids.contains(&String::from("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb")));

        // Check need IDs
        #[cfg(feature = "std")]
        need_ids.sort();
        assert_eq!(
            need_ids,
            vec![
                String::from("11111111111111111111111111111111"),
                String::from("22222222222222222222222222222222"),
                String::from("33333333333333333333333333333333"),
                String::from("cccccccccccccccccccccccccccccccc"),
            ]
        )
    }

    #[test]
    fn test_invalid_id_size() {
        assert_eq!(Negentropy::new(33, None).unwrap_err(), Error::InvalidIdSize);

        let mut client = Negentropy::new(16, None).unwrap();
        assert_eq!(
            client.add_item(0, "abcdef").unwrap_err(),
            Error::IdSizeNotMatch
        );
    }
}
