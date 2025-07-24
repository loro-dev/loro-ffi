use std::{cmp::Ordering, collections::HashMap, sync::RwLock};

use loro::{IdSpan, LoroResult, PeerID, ID};

use crate::CounterSpan;

pub struct VersionVector(RwLock<loro::VersionVector>);

impl Default for VersionVector {
    fn default() -> Self {
        Self::new()
    }
}

impl VersionVector {
    pub fn new() -> Self {
        Self(RwLock::new(loro::VersionVector::default()))
    }

    pub fn diff(&self, rhs: &Self) -> VersionVectorDiff {
        self.0.read().unwrap().diff(&rhs.0.read().unwrap()).into()
    }

    pub fn get_last(&self, peer: PeerID) -> Option<i32> {
        self.0.read().unwrap().get_last(peer)
    }

    pub fn set_last(&self, id: ID) {
        self.0.write().unwrap().set_last(id);
    }

    pub fn set_end(&self, id: ID) {
        self.0.write().unwrap().set_end(id);
    }

    pub fn get_missing_span(&self, target: &Self) -> Vec<IdSpan> {
        self.0
            .read()
            .unwrap()
            .get_missing_span(&target.0.read().unwrap())
    }

    pub fn merge(&self, other: &VersionVector) {
        self.0.write().unwrap().merge(&other.0.read().unwrap())
    }

    pub fn includes_vv(&self, other: &VersionVector) -> bool {
        self.0.read().unwrap().includes_vv(&other.0.read().unwrap())
    }

    pub fn includes_id(&self, id: ID) -> bool {
        self.0.read().unwrap().includes_id(id)
    }

    pub fn intersect_span(&self, target: IdSpan) -> Option<CounterSpan> {
        self.0.read().unwrap().intersect_span(target)
    }

    pub fn extend_to_include_vv(&self, other: &VersionVector) {
        self.0
            .write()
            .unwrap()
            .extend_to_include_vv(other.0.read().unwrap().iter());
    }

    pub fn partial_cmp(&self, other: &VersionVector) -> Option<Ordering> {
        self.0.read().unwrap().partial_cmp(&other.0.read().unwrap())
    }

    pub fn encode(&self) -> Vec<u8> {
        self.0.read().unwrap().encode()
    }

    pub fn decode(bytes: &[u8]) -> LoroResult<Self> {
        let ans = Self(RwLock::new(loro::VersionVector::decode(bytes)?));
        Ok(ans)
    }

    pub fn to_hashmap(&self) -> HashMap<u64, i32> {
        self.0
            .read()
            .unwrap()
            .iter()
            .map(|(id, version)| (*id, *version))
            .collect()
    }

    pub fn try_update_last(&self, id: ID) -> bool {
        self.0.write().unwrap().try_update_last(id)
    }
}

impl PartialEq for VersionVector {
    fn eq(&self, other: &Self) -> bool {
        self.0.read().unwrap().eq(&other.0.read().unwrap())
    }
}

impl Eq for VersionVector {}

#[derive(Debug)]
pub struct Frontiers(loro::Frontiers);

impl Frontiers {
    pub fn new() -> Self {
        Self(loro::Frontiers::default())
    }

    pub fn from_id(id: ID) -> Self {
        Self(loro::Frontiers::from(id))
    }

    pub fn from_ids(ids: Vec<ID>) -> Self {
        Self(loro::Frontiers::from(ids))
    }

    pub fn encode(&self) -> Vec<u8> {
        self.0.encode()
    }

    pub fn decode(bytes: &[u8]) -> LoroResult<Self> {
        let ans = Self(loro::Frontiers::decode(bytes)?);
        Ok(ans)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn to_vec(&self) -> Vec<ID> {
        self.0.to_vec()
    }
}

impl PartialEq for Frontiers {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl Eq for Frontiers {}

impl Default for Frontiers {
    fn default() -> Self {
        Self::new()
    }
}

pub struct VersionVectorDiff {
    /// need to add these spans to move from right to left
    pub retreat: HashMap<PeerID, CounterSpan>,
    /// need to add these spans to move from left to right
    pub forward: HashMap<PeerID, CounterSpan>,
}

pub struct VersionRangeItem {
    pub peer: PeerID,
    pub start: i32,
    pub end: i32,
}

pub struct VersionRange(RwLock<loro::VersionRange>);

impl VersionRange {
    pub fn new() -> Self {
        Self(RwLock::new(loro::VersionRange::new()))
    }

    pub fn from_vv(vv: &VersionVector) -> Self {
        let loro_vv: loro::VersionVector = vv.into();
        Self(RwLock::new(loro::VersionRange::from_vv(&loro_vv)))
    }

    pub fn clear(&self) {
        self.0.write().unwrap().clear();
    }

    pub fn get(&self, peer: PeerID) -> Option<CounterSpan> {
        self.0
            .read()
            .unwrap()
            .get(&peer)
            .map(|(start, end)| CounterSpan::new(*start, *end))
    }

    pub fn insert(&self, peer: PeerID, start: i32, end: i32) {
        self.0.write().unwrap().insert(peer, start, end);
    }

    pub fn contains_ops_between(&self, vv_a: &VersionVector, vv_b: &VersionVector) -> bool {
        let loro_vv_a: loro::VersionVector = vv_a.into();
        let loro_vv_b: loro::VersionVector = vv_b.into();
        self.0
            .read()
            .unwrap()
            .contains_ops_between(&loro_vv_a, &loro_vv_b)
    }

    pub fn has_overlap_with(&self, span: IdSpan) -> bool {
        self.0.read().unwrap().has_overlap_with(span)
    }

    pub fn contains_id(&self, id: ID) -> bool {
        self.0.read().unwrap().contains_id(id)
    }

    pub fn contains_id_span(&self, span: IdSpan) -> bool {
        self.0.read().unwrap().contains_id_span(span)
    }

    pub fn extends_to_include_id_span(&self, span: IdSpan) {
        self.0.write().unwrap().extends_to_include_id_span(span);
    }

    pub fn is_empty(&self) -> bool {
        self.0.read().unwrap().is_empty()
    }

    pub fn get_peers(&self) -> Vec<PeerID> {
        self.0
            .read()
            .unwrap()
            .iter()
            .map(|(peer, _)| *peer)
            .collect()
    }

    pub fn get_all_ranges(&self) -> Vec<VersionRangeItem> {
        self.0
            .read()
            .unwrap()
            .iter()
            .map(|(peer, (start, end))| VersionRangeItem {
                peer: *peer,
                start: *start,
                end: *end,
            })
            .collect()
    }
}

impl Default for VersionRange {
    fn default() -> Self {
        Self::new()
    }
}

impl From<loro::VersionRange> for VersionRange {
    fn from(value: loro::VersionRange) -> Self {
        Self(RwLock::new(value))
    }
}

impl From<VersionRange> for loro::VersionRange {
    fn from(value: VersionRange) -> Self {
        value.0.into_inner().unwrap()
    }
}

impl From<&VersionRange> for loro::VersionRange {
    fn from(value: &VersionRange) -> Self {
        value.0.read().unwrap().clone()
    }
}

impl From<loro::VersionVectorDiff> for VersionVectorDiff {
    fn from(value: loro::VersionVectorDiff) -> Self {
        Self {
            retreat: value.retreat.into_iter().collect(),
            forward: value.forward.into_iter().collect(),
        }
    }
}

impl From<VersionVector> for loro::VersionVector {
    fn from(value: VersionVector) -> Self {
        value.0.into_inner().unwrap()
    }
}

impl From<&VersionVector> for loro::VersionVector {
    fn from(value: &VersionVector) -> Self {
        value.0.read().unwrap().clone()
    }
}

impl From<loro::VersionVector> for VersionVector {
    fn from(value: loro::VersionVector) -> Self {
        Self(RwLock::new(value))
    }
}

impl From<loro::Frontiers> for Frontiers {
    fn from(value: loro::Frontiers) -> Self {
        Self(value)
    }
}

impl From<Frontiers> for loro::Frontiers {
    fn from(value: Frontiers) -> Self {
        value.0
    }
}

impl From<&Frontiers> for loro::Frontiers {
    fn from(value: &Frontiers) -> Self {
        value.0.clone()
    }
}
