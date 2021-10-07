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

//! Collection of structures that keep the SDK independent from the core

use crate::ecdsa;
use serde::{Deserialize, Serialize};

/// Structure passet from the host to the wasm smart contracts.
///
/// **WARNING:** ANY MODIFICATION CAN BREAK COMPATIBILITY WITH THE CORE.
#[derive(Serialize, Deserialize)]
pub struct AppInput<'a> {
    /// Nested call depth.
    pub depth: u16,
    /// Network identifier (from Tx)
    pub network: &'a str,
    /// Identifier of the account that the method is targeting.
    pub owner: &'a str,
    /// Caller's identifier.
    pub caller: &'a str,
    /// Method name.
    pub method: &'a str,
    /// Original transaction submitter (from Tx)
    pub origin: &'a str,
}

/// Structure returned from the wasm smart contracts to the host.
///
/// **WARNING:** ANY MODIFICATION CAN BREAK COMPATIBILITY WITH THE CORE.
#[derive(Serialize, Deserialize)]
pub struct AppOutput<'a> {
    /// Contract execution status.
    pub success: bool,
    /// Execution result data of success. Error string on failure.
    #[serde(with = "serde_bytes")]
    pub data: &'a [u8],
}

/// Helper macro to allow serialization of named unit variants by name.
#[warn(dead_code)]
#[macro_export]
macro_rules! named_unit_variant {
    ($variant:ident) => {
        pub(crate) mod $variant {
            #[allow(dead_code)]
            pub(crate) fn serialize<S>(serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                serializer.serialize_str(stringify!($variant))
            }
            #[allow(dead_code)]
            pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<(), D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct V;
                impl<'de> serde::de::Visitor<'de> for V {
                    type Value = ();
                    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                        f.write_str(concat!("\"", stringify!($variant), "\""))
                    }
                    fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Self::Value, E> {
                        if value == stringify!($variant) {
                            Ok(())
                        } else {
                            Err(E::invalid_value(serde::de::Unexpected::Str(value), &self))
                        }
                    }
                }
                deserializer.deserialize_str(V)
            }
        }
    };
}

/// PublicKey type
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(tag = "type")]
pub enum PublicKey {
    #[serde(rename = "ecdsa")]
    Ecdsa(ecdsa::PublicKey),
}
