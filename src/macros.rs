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

//! Rust SDK macros
//!
//! See the Macros section of the crate documentation

/// Helper macro to construct the application entry point.
///
/// The `input` and `output` values are encoded using MessagePack format.
#[macro_export]
macro_rules! app_export {
    ($($fun:expr),*) => {
        #[doc(hidden)]
        #[no_mangle]
        /// Entry point of the smart contract calls
        fn app_run(ctx: $crate::AppContext, buf: &[u8]) -> Result<Vec<u8>, $crate::WasmError> {
            use $crate::{Serializable, Deserializable};
            match ctx.method {
                $(
                    stringify!($fun) => {
                        let input = Deserializable::deserialize(buf)?;
                        let output = $fun(ctx, input)?;
                        Serializable::serialize(&output)
                    },
                )*
                _ => Err($crate::WasmError::new(&format!("method `{}` not found", ctx.method))),
            }
        }
    };
}

/// Store account data in message pack format.
///
/// The `value` shall implement `Serialize` trait.
#[macro_export]
macro_rules! store_account_data_mp {
    (
        $(#[$meta:meta])*
        $id:expr, $value:expr) => {
        $crate::rmp_serialize($value).and_then(|buf| Ok($crate::store_data($id, &buf)))
    };
}

/// Load account data stored in message pack format.
///
/// The target `value` shall implement `Deserialize` trait.
/// Note: this is usable only if the target data structure doesn't contain any reference.
#[macro_export]
macro_rules! load_account_data_mp {
    ($id:expr) => {
        $crate::rmp_deserialize(&$crate::load_data($id));
    };
}

#[macro_export]
/// Get the content of a Value field as a valid Json type
///
/// Value types: object, u64, u32, str, ...
macro_rules! get_value_as {
    ($value:expr, $index:expr, $vtype:ident) => {
        $value
            .get(&value!($index))
            .ok_or_else(|| {
                $crate::WasmError::new(&format!("`{}::{}` not found", stringify!($value), $index))
            })
            .and_then(|value| {
                value.$vtype().ok_or_else(|| {
                    $crate::WasmError::new(&format!(
                        "`{}::{}` has bad type",
                        stringify!($value),
                        $index
                    ))
                })
            })
    };
}

/// Get an `object` reference from a `json_serde::Value` by key.
#[macro_export]
macro_rules! get_value_as_object {
    ($value:expr, $index:expr) => {
        $crate::get_value_as!($value, $index, as_object)
    };
}

/// Get a `str` reference from a `json_serde::Value` by key.
#[macro_export]
macro_rules! get_value_as_str {
    ($value:expr, $index:expr) => {
        $crate::get_value_as!($value, $index, as_str)
    };
}

/// Get a `u64` reference from a `json_serde::Value` by key.
#[macro_export]
macro_rules! get_value_as_u64 {
    ($value:expr, $index:expr) => {
        $crate::get_value_as!($value, $index, as_u64)
    };
}

/// Get an `array` reference from a `json_serde::Valu` by key.
#[macro_export]
macro_rules! get_value_as_array {
    ($value:expr, $index:expr) => {
        $crate::get_value_as!($value, $index, as_array)
    };
}

/// Helper macro around sdk logging facility to allow format strings.
#[macro_export]
macro_rules! log {
    ($msg:expr) => {
        $crate::log($msg);
    };
    ($fmt:expr, $($args:expr),*) => {
        let msg = format!($fmt, $($args),*);
        $crate::log(&msg);
    };
}
