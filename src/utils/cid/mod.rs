// Copyright 2019-2023 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use cid::{
    multihash::{self, Code, Code::Blake2b256, MultihashDigest},
    Cid, Version,
};
use fvm_ipld_encoding::{Error, DAG_CBOR};
#[cfg(test)]
use quickcheck::Arbitrary;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Extension methods for constructing `dag-cbor` [Cid]
pub trait CidCborExt {
    /// Default CID builder for Filecoin
    ///
    /// - The default codec is [`fvm_ipld_encoding::DAG_CBOR`]
    /// - The default hash function is 256 bit BLAKE2b
    ///
    /// This matches [`abi.CidBuilder`](https://github.com/filecoin-project/go-state-types/blob/master/abi/cid.go#L49) in go
    fn from_cbor_blake2b256<S: serde::ser::Serialize>(obj: &S) -> Result<Cid, Error> {
        let bytes = fvm_ipld_encoding::to_vec(obj)?;
        Ok(Cid::new_v1(
            fvm_ipld_encoding::DAG_CBOR,
            Code::Blake2b256.digest(&bytes),
        ))
    }
}

impl CidCborExt for Cid {}

pub const BLAKE2B256_SIZE: usize = 32;

/// `SmallCid` encapsulates an enumeration of known CID types that are used in the Filecoin blockchain. CIDs
/// contain a significant amount of static data (such as version, codec, hash identifier, hash
/// length). This static data represented by a single tag in the `enum`.
///
/// Nearly all Filecoin CIDs are `V1`,`DagCbor` encoded, and hashed with `Blake2b256` (which has a hash
/// length of 256 bits). Naively representing such a CID requires 96 bytes but `SmallCid` does it in
/// only 40 bytes. If other types of CID become popular, they can be added to the `SmallCid`
/// structure.
///
/// The `Generic` variant is used for CIDs that do not fit into the other variants.
/// These variants are used for optimizing storage of CIDs in the `FrozenCids` structure.
///
/// The contained [`SmallCidInner`] is guaranteed to be canonical.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SmallCid(SmallCidInner);

impl SmallCid {
    pub fn cid(&self) -> Cid {
        match &self.0 {
            SmallCidInner::Other(cid) => **cid,
            SmallCidInner::V1DagCborBlake2b(digest) => Cid::new_v1(
                DAG_CBOR,
                multihash::Multihash::wrap(Blake2b256.into(), digest)
                    .expect("failed to convert Blake2b digest to Multihash for creation of V1 DAG-CBOR Blake2b CID"),
            ),
        }
    }
}

impl SmallCidInner {
    /// [`SmallCidInner::Other`] should not contain a CID which could be represented by more specialized variants.
    fn canonical(cid: Cid) -> SmallCidInner {
        if cid.version() == Version::V1 && cid.codec() == DAG_CBOR {
            if let Ok(small_hash) = cid.hash().resize() {
                let (code, bytes, size) = small_hash.into_inner();
                if code == u64::from(Code::Blake2b256) && size as usize == BLAKE2B256_SIZE {
                    return SmallCidInner::V1DagCborBlake2b(bytes);
                }
            }
        }
        SmallCidInner::Other(Box::new(cid))
    }
}

/// No guarantees are made about canonicalization with this struct
/// That is, you may have a [`Self::Other`] variant which could be represented as a [`Self::V1DagCborBlake2b`]
/// (typically as a result of calling `quickcheck::Arbitrary::arbitrary`)
#[cfg_attr(test, derive(derive_quickcheck_arbitrary::Arbitrary))]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum SmallCidInner {
    Other(Box<Cid>),
    V1DagCborBlake2b(
        #[cfg_attr(test, arbitrary(gen(|g: &mut quickcheck::Gen| std::array::from_fn(|_ix| Arbitrary::arbitrary(g)))))]
         [u8; BLAKE2B256_SIZE],
    ),
}

impl Serialize for SmallCid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.cid().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for SmallCid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Self::from(Cid::deserialize(deserializer)?))
    }
}

impl From<Cid> for SmallCid {
    fn from(cid: Cid) -> Self {
        SmallCid(SmallCidInner::canonical(cid))
    }
}

impl From<SmallCid> for Cid {
    fn from(variant: SmallCid) -> Self {
        Cid::from(&variant)
    }
}

impl From<&SmallCid> for Cid {
    fn from(variant: &SmallCid) -> Self {
        variant.cid()
    }
}

#[cfg(test)]
mod tests {
    use super::SmallCid;
    use super::*;
    use crate::db::MemoryDB;
    use crate::utils::db::CborStoreExt;
    use anyhow::*;
    use cid::{
        multihash::{Code, MultihashDigest},
        Cid,
    };
    use fvm_ipld_encoding::DAG_CBOR;
    use quickcheck_macros::quickcheck;
    use std::mem::size_of;

    impl Arbitrary for SmallCid {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            SmallCid(SmallCidInner::canonical(Cid::arbitrary(g)))
        }
    }

    #[quickcheck]
    fn test_cid_cbor_ext(s: String) -> Result<()> {
        let cid1 = Cid::from_cbor_blake2b256(&s)?;
        let cid2 = {
            let store = MemoryDB::default();
            store.put_cbor_default(&s)?
        };
        ensure!(cid1 == cid2);

        Ok(())
    }

    // If this stops being true, please update the documentation above.
    #[test]
    fn cid_size_assumption() {
        assert_eq!(size_of::<Cid>(), 96);
    }

    // If this stops being true, please update the BLAKE2B256_SIZE constant.
    #[test]
    fn blake_size_assumption() {
        assert_eq!(
            Code::Blake2b256.digest(&[]).size() as usize,
            super::BLAKE2B256_SIZE
        );
    }

    #[test]
    fn known_v1_blake2b() {
        let cid = Cid::new(
            cid::Version::V1,
            DAG_CBOR,
            Code::Blake2b256.digest("blake".as_bytes()),
        )
        .unwrap();
        assert!(matches!(
            cid.try_into().unwrap(),
            SmallCid(SmallCidInner::V1DagCborBlake2b(_))
        ));
    }

    // If this test fails, the default encoding is no longer v1+dagcbor+blake2b. Consider adding the new default
    // CID type to `SmallCid`.
    #[test]
    fn default_is_v1_dagcbor() {
        let cid = MemoryDB::default().put_cbor_default(&()).unwrap();
        assert!(matches!(
            cid.try_into().unwrap(),
            SmallCid(SmallCidInner::V1DagCborBlake2b(_))
        ));
    }
}
