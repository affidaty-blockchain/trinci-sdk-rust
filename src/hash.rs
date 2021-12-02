// This file is part of TRINCI.
//
// Copyright (C) 2021 Affidaty Spa.
//
// TRINCI is free software: you can redistribute it and/or modify it under
// the terms of the GNU Affero General Public License as published by the
// Free Software Foundation, either version 3 of the License, or (at your
// option) any later version.
//
// TRINCI is distributed in the hope that it will be useful, but WITHOUT
// ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or
// FITNESS FOR A PARTICULAR PURPOSE. See the GNU Affero General Public License
// for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with TRINCI. If not, see <https://www.gnu.org/licenses/>.

//! Opaque cryptographic secure hash used by the overall project.
//!
//! Current implementation uses SHA-256.
//!
//! The serialization uses [Multihash](https://multiformats.io/multihash) format
//! to keep a door opened for future extensions.
//!
//! Complete multihash table lives
//! [here](https://github.com/multiformats/multicodec/blob/master/table.csv).

use sha2::{Digest, Sha256};

/// Available hash algorithms.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum HashAlgorithm {
    Identity,
    Sha256,
}

impl Default for HashAlgorithm {
    fn default() -> Self {
        HashAlgorithm::Identity
    }
}

/// Multihash tag for Identity
const MULTIHASH_TYPE_IDENTITY: u8 = 0x00;
/// Multihash SHA-256 type
const MULTIHASH_TYPE_SHA256: u8 = 0x12;

/// Max length of multihash value.
const MULTIHASH_VALUE_LEN_MAX: usize = 32;

/// Max serialized length.
const MULTIHASH_BYTES_LEN_MAX: usize = 2 + MULTIHASH_VALUE_LEN_MAX;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct Hash(pub [u8; MULTIHASH_BYTES_LEN_MAX]);

impl Default for Hash {
    fn default() -> Self {
        // Implicitly sets algorithm to "identity" and length to 0
        Hash([0; MULTIHASH_BYTES_LEN_MAX])
    }
}

impl Hash {
    // Creates a new instance by wrapping precomputed hash bytes.
    pub fn new(alg: HashAlgorithm, bytes: &[u8]) -> Self {
        let mut hash = Hash::default();
        let hash_len = bytes.len();

        let hash_alg = match alg {
            HashAlgorithm::Identity => MULTIHASH_TYPE_IDENTITY,
            HashAlgorithm::Sha256 => MULTIHASH_TYPE_SHA256,
        };
        hash.0[0] = hash_alg;
        hash.0[1] = hash_len as u8;
        hash.0[2..(2 + hash_len)].copy_from_slice(bytes);
        hash
    }

    /// Compute hash from arbitrary data.
    pub fn from_data(alg: HashAlgorithm, data: &[u8]) -> Self {
        match alg {
            HashAlgorithm::Sha256 => {
                let mut hasher = Sha256::new();
                hasher.update(data);
                let digest = hasher.finalize();
                Hash::new(alg, digest.as_ref())
            }
            HashAlgorithm::Identity => Hash::new(alg, data),
        }
    }
}

/// A trait for types that can be hashed.
pub trait Hashable {
    /// Hash using the chosen hash algorithm.
    fn hash(&self, alg: HashAlgorithm) -> Hash;
}
