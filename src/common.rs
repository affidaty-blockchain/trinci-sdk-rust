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

//! Commons utilities and traits

use crate::core::AppInput;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

/// Wasm application execution context.
/// Contains additional data that may be useful to the method.
pub type AppContext<'a> = AppInput<'a>;

/// Wasm application method result type.
pub type WasmResult<T> = std::result::Result<T, WasmError>;

/// Project-wide error type.
/// Contains a kind enumerate and a `source` to identify the subsystem that may
/// have propageted the error.
#[derive(Debug)]
pub struct WasmError(String);

/// Display support.
impl Display for WasmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Standard error trait support.
impl std::error::Error for WasmError {}

impl WasmError {
    /// Constructor.
    pub fn new(msg: &str) -> WasmError {
        WasmError(msg.to_owned())
    }
}

/// Compact representation of a wasm slice components.
/// In wasm an address and a length are two i32.
pub(crate) type WasmSlice = u64;

/// Combines two i32 into one u64.
fn wslice_create(offset: i32, length: i32) -> WasmSlice {
    ((offset as u64) << 32) | (length as u64) & 0x00000000ffffffff
}

/// Splits one u64 into two i32.
fn wslice_split(wslice: WasmSlice) -> (i32, i32) {
    (
        ((wslice & 0xffffffff00000000) >> 32) as i32,
        (wslice & 0x00000000ffffffff) as i32,
    )
}

#[cfg(target_arch = "wasm32")]
/// Returns the buffer pointer in the wasm memory
pub(crate) fn slice_to_mem(buf: &[u8]) -> i32 {
    buf.as_ptr() as i32
}

#[cfg(not(target_arch = "wasm32"))]
/// Returns the buffer pointer in the mocked wasm memory
pub(crate) fn slice_to_mem(buf: &[u8]) -> i32 {
    crate::not_wasm::write_mem(buf)
}

#[cfg(target_arch = "wasm32")]
/// Create a WasmSlice from a memory buffer in the wasm memory
pub(crate) fn slice_to_wslice(buf: &[u8]) -> WasmSlice {
    wslice_create(buf.as_ptr() as i32, buf.len() as i32)
}

#[cfg(not(target_arch = "wasm32"))]
/// Create a WasmSlice from a memory buffer in the mocked wasm memory
pub(crate) fn slice_to_wslice(buf: &[u8]) -> WasmSlice {
    let offset = crate::not_wasm::write_mem(buf);
    wslice_create(offset, buf.len() as i32)
}

#[cfg(target_arch = "wasm32")]
/// Load data from the wasm memory
pub(crate) fn slice_from_mem<'a>(offset: i32, length: i32) -> &'a [u8] {
    unsafe { std::slice::from_raw_parts(offset as usize as *mut u8, length as usize) }
}

#[cfg(not(target_arch = "wasm32"))]
/// Load data from the mocked wasm memory
pub(crate) fn slice_from_mem<'a>(offset: i32, length: i32) -> &'a [u8] {
    let addr = offset as usize + crate::not_wasm::memory_base();
    unsafe { std::slice::from_raw_parts(addr as *mut u8, length as usize) }
}

#[cfg(target_arch = "wasm32")]
/// Create a slice in the wasm memory from a WasmSlice structure
pub(crate) fn slice_from_wslice<'a>(wslice: WasmSlice) -> &'a [u8] {
    let (offset, length) = wslice_split(wslice);
    unsafe { std::slice::from_raw_parts(offset as usize as *mut u8, length as usize) }
}

#[cfg(not(target_arch = "wasm32"))]
/// Create a slice in the mocked wasm memory from a WasmSlice structure
pub(crate) fn slice_from_wslice<'a>(wslice: WasmSlice) -> &'a [u8] {
    let (offset, length) = wslice_split(wslice);
    let addr = offset as usize + crate::not_wasm::memory_base();
    unsafe { std::slice::from_raw_parts(addr as *mut u8, length as usize) }
}

/// Serialize a type implementing `Serialize` trait using MessagePack format with named keys.
pub fn rmp_serialize_named<T>(val: &T) -> WasmResult<Vec<u8>>
where
    T: Serialize,
{
    rmp_serde::to_vec_named(val).map_err(|_err| WasmError::new("serialization failure"))
}

/// Serialize a type implementing `Serialize` trait using MessagePack format.
pub fn rmp_serialize<T>(val: &T) -> WasmResult<Vec<u8>>
where
    T: Serialize,
{
    rmp_serde::to_vec(val).map_err(|_err| WasmError::new("serialization failure"))
}

/// Serialize a type implementing `Deserialize` trait using MessagePack format.
pub fn rmp_deserialize<'a, T>(buf: &'a [u8]) -> WasmResult<T>
where
    T: Deserialize<'a>,
{
    rmp_serde::from_slice(buf).map_err(|_err| WasmError::new("deserialization failure"))
}

/// Value that has been already packed, thus it doesn't require further
/// processing and shall be taken "as-is".
#[derive(Default, Debug)]
pub struct PackedValue(pub Vec<u8>);

impl std::ops::Deref for PackedValue {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Messagepack serialization trait
pub trait Serializable: Sized {
    fn serialize(&self) -> WasmResult<Vec<u8>>;
}

impl<T: Serialize> Serializable for T {
    fn serialize(&self) -> WasmResult<Vec<u8>> {
        rmp_serialize_named(self)
    }
}

impl Serializable for PackedValue {
    fn serialize(&self) -> WasmResult<Vec<u8>> {
        Ok(self.0.clone())
    }
}

/// Messagepack deserialization trait
pub trait Deserializable<'a>: Sized {
    fn deserialize(buf: &'a [u8]) -> WasmResult<Self>;
}

impl<'a, T: Deserialize<'a>> Deserializable<'a> for T {
    fn deserialize(buf: &'a [u8]) -> WasmResult<Self> {
        rmp_deserialize(buf)
    }
}

impl Deserializable<'_> for PackedValue {
    fn deserialize(buf: &'_ [u8]) -> WasmResult<Self> {
        Ok(PackedValue(buf.to_vec()))
    }
}
