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

#[macro_use]
pub mod macros;

pub mod common;
pub mod host_wrap;
pub mod tai;
pub mod value;

mod export;

// TEMPORARY MODULES :: BEGIN
pub mod core;
pub mod ecdsa;
pub mod hash;
// TEMPORARY MODULES :: END

pub use serde_value::{value, Value};

pub use common::{
    divide, rmp_deserialize, rmp_serialize, rmp_serialize_named, AppContext, Deserializable,
    PackedValue, Serializable, WasmError, WasmResult,
};
pub use host_wrap::{
    asset_balance, asset_lock, asset_transfer, call, emit_data, get_account_contract,
    get_data_keys, is_callable, load_asset, load_asset_typed, load_data, log, remove_data, s_call,
    sha256, store_asset, store_asset_typed, store_data, verify,
};

// Testing helpers on not wasm environments.
#[cfg(not(target_arch = "wasm32"))]
pub mod not_wasm;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const VERSION_MAJOR: &str = env!("CARGO_PKG_VERSION_MAJOR");
pub const VERSION_MINOR: &str = env!("CARGO_PKG_VERSION_MINOR");
pub const VERSION_PATCH: &str = env!("CARGO_PKG_VERSION_PATCH");
pub const VERSION_PRE: &str = env!("CARGO_PKG_VERSION_PRE");
