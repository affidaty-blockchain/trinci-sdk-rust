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

//! Ecdsa utilities for the SDK

use serde::{Deserialize, Serialize};

crate::named_unit_variant!(secp384r1);

/// ECDSA Curve
///
/// **WARNING:** ANY MODIFICATION CAN BREAK COMPATIBILITY WITH THE CORE.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
#[serde(untagged)]
pub enum CurveId {
    #[serde(with = "secp384r1")]
    Secp384R1,
}

/// ECDSA PublicKey
///
/// **WARNING:** ANY MODIFICATION CAN BREAK COMPATIBILITY WITH THE CORE.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct PublicKey {
    pub curve: CurveId,
    #[serde(with = "serde_bytes")]
    pub value: Vec<u8>,
}
