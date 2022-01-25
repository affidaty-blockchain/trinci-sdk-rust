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

//! Host functions wrapper

use crate::{
    common::*,
    core::{AppOutput, PublicKey},
    tai::{AssetLockArgs, AssetTransferArgs, LockType},
};

use serde::{de::DeserializeOwned, Serialize};

/// Host functions imported
extern "C" {
    /// Raw log host funtion
    fn hf_log(msg_addr: i32, msg_size: i32);

    /// Raw emit host function
    fn hf_emit(
        event_name_addr: i32,
        event_name_size: i32,
        event_data_addr: i32,
        event_data_size: i32,
    );

    /// Raw get_keys host function
    fn hf_get_keys(pattern_addr: i32, pattern_size: i32) -> WasmSlice;

    /// Raw store_data host funtion
    fn hf_store_data(key_addr: i32, key_size: i32, data_addr: i32, data_size: i32);

    /// Raw load_data host funtion
    fn hf_load_data(key_addr: i32, key_size: i32) -> WasmSlice;

    /// Raw remove_data host funtion
    fn hf_remove_data(key_addr: i32, key_size: i32);

    /// Raw load asset host funtion
    fn hf_load_asset(id_addr: i32, id_size: i32) -> WasmSlice;

    /// Raw store_asset host funtion
    fn hf_store_asset(id_addr: i32, id_size: i32, value_addr: i32, value_size: i32);

    /// Raw get account contract host funtion
    fn hf_get_account_contract(id_addr: i32, id_size: i32) -> WasmSlice;

    /// Raw verify host funtion
    fn hf_verify(
        pk_addr: i32,
        pk_size: i32,
        data_addr: i32,
        data_size: i32,
        sign_addr: i32,
        sign_size: i32,
    ) -> i32;

    /// Raw call host funtion
    fn hf_call(
        account_addr: i32,
        account_size: i32,
        method_addr: i32,
        method_size: i32,
        data_addr: i32,
        data_size: i32,
    ) -> WasmSlice;

    /// Sha256 host function
    fn hf_sha256(data_addr: i32, data_size: i32) -> WasmSlice;

}

/// Logging facility for smart contracts.
pub fn log(msg: &str) {
    let msg_addr = slice_to_mem(msg.as_bytes());
    unsafe {
        hf_log(msg_addr, msg.len() as i32);
    }
}

/// Notification facility for smart contracts.
pub fn emit_data(event_name: &str, event_data: &[u8]) {
    let event_name_addr = slice_to_mem(event_name.as_bytes());
    let event_data_addr = slice_to_mem(event_data);
    unsafe {
        hf_emit(
            event_name_addr,
            event_name.len() as i32,
            event_data_addr,
            event_data.len() as i32,
        );
    }
}

/// Load account data associated to the given key.
pub fn load_data(key: &str) -> Vec<u8> {
    let key_addr = slice_to_mem(key.as_bytes());
    let wslice = unsafe { hf_load_data(key_addr, key.len() as i32) };
    slice_from_wslice(wslice).to_vec()
}

/// Get the account contract to the given account id
pub fn get_account_contract(id: &str) -> Vec<u8> {
    let id_addr = slice_to_mem(id.as_bytes());
    let wslice = unsafe { hf_get_account_contract(id_addr, id.len() as i32) };
    slice_from_wslice(wslice).to_vec()
}

/// Get the account keys.
pub fn get_data_keys(pattern: &str) -> WasmResult<Vec<String>> {
    let pattern_addr = slice_to_mem(pattern.as_bytes());
    let wslice = unsafe { hf_get_keys(pattern_addr, pattern.len() as i32) };
    let buf = slice_from_wslice(wslice).to_vec();
    let res: AppOutput = rmp_deserialize(&buf)?;
    match res.success {
        true => rmp_deserialize::<Vec<String>>(res.data),
        false => Err(WasmError::new(String::from_utf8_lossy(res.data).as_ref())),
    }
}

/// Store account data associated to the given key.
pub fn store_data(key: &str, buf: &[u8]) {
    let data_addr = slice_to_mem(buf);
    let key_addr = slice_to_mem(key.as_bytes());
    unsafe { hf_store_data(key_addr, key.len() as i32, data_addr, buf.len() as i32) };
}

/// Remove account data associated to the given key.
pub fn remove_data(key: &str) {
    let key_addr = slice_to_mem(key.as_bytes());
    unsafe { hf_remove_data(key_addr, key.len() as i32) };
}

/// Load asset with the given asset id from the current account as byte array.
pub fn load_asset(id: &str) -> Vec<u8> {
    let id_addr = slice_to_mem(id.as_bytes());
    let wslice = unsafe { hf_load_asset(id_addr, id.len() as i32) };
    slice_from_wslice(wslice).to_vec()
}

/// Store asset with the given asset id in the current account as byte array .
pub fn store_asset(id: &str, value: &[u8]) {
    let id_addr = slice_to_mem(id.as_bytes());
    let value_addr = slice_to_mem(value);
    unsafe { hf_store_asset(id_addr, id.len() as i32, value_addr, value.len() as i32) };
}

/// Verify the signature of the given data by the given pk and algorithm
pub fn verify(pk: &PublicKey, data: &[u8], sign: &[u8]) -> bool {
    let pk = match rmp_serialize(&pk) {
        Ok(val) => val,
        Err(_) => return false,
    };
    let pk_addr = slice_to_mem(&pk);
    let data_addr = slice_to_mem(data);
    let sign_addr = slice_to_mem(sign);

    unsafe {
        hf_verify(
            pk_addr,
            pk.len() as i32,
            data_addr,
            data.len() as i32,
            sign_addr,
            sign.len() as i32,
        ) == 1
    }
}

/// Calculates the Sha256 hash of the data
pub fn sha256(data: &[u8]) -> Vec<u8> {
    let data_addr = slice_to_mem(data);
    let wslice = unsafe { hf_sha256(data_addr, data.len() as i32) };
    slice_from_wslice(wslice).to_vec()
}

/// Call a method of an arbitrary smart contract passing the data as argument
pub fn call(account: &str, method: &str, data: &[u8]) -> WasmResult<Vec<u8>> {
    let account_addr = slice_to_mem(account.as_bytes());
    let method_addr = slice_to_mem(method.as_bytes());
    let data_addr = slice_to_mem(data);
    let wslice = unsafe {
        hf_call(
            account_addr,
            account.len() as i32,
            method_addr,
            method.len() as i32,
            data_addr,
            data.len() as i32,
        )
    };
    let buf = slice_from_wslice(wslice);
    let result: AppOutput = rmp_deserialize(buf)?;
    match result.success {
        true => Ok(result.data.to_owned()),
        false => {
            let msg = String::from_utf8_lossy(result.data);
            Err(WasmError::new(&msg))
        }
    }
}

/// Get account balance for a given asset.
///
/// This is an helper function over the lower level `call(asset_id, "balance", args)`.
pub fn asset_balance(asset: &str) -> WasmResult<u64> {
    call(asset, "balance", &[]).map(|buf| rmp_deserialize(&buf).unwrap_or_default())
}

/// Transfer an amount of asset units to a destination account.
///
/// This is an helper function over the lower level `call(asset_id, "transfer", args)`.
pub fn asset_transfer(from: &str, to: &str, asset: &str, units: u64) -> WasmResult<()> {
    let data = rmp_serialize(&AssetTransferArgs { from, to, units })?;
    call(asset, "transfer", &data).map(|_buf| ())
}

/// Lock/Unlock the asset.
///
/// This is an helper function over the lower level `call(asset_id, "lock", true/false)`.
pub fn asset_lock(asset: &str, to: &str, value: LockType) -> WasmResult<()> {
    let data = rmp_serialize(&AssetLockArgs { to, lock: value })?;
    call(asset, "lock", &data).map(|_buf| ())
}

/// Load asset with the given asset id from the current account
/// and tries to convert it into a type.
pub fn load_asset_typed<T: DeserializeOwned + Default>(id: &str) -> T {
    let buf = load_asset(id);
    rmp_deserialize(&buf).unwrap_or_default()
}

/// Store the typed asset with the given asset id in the current account.
pub fn store_asset_typed<T: Serialize>(id: &str, value: T) {
    let buf = rmp_serialize(&value).unwrap();
    store_asset(id, &buf);
}
