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

//! Collection of support structures and functions when the smart contract is not run within the
//! wasm machine.

use crate::{
    common::*,
    core::{AppOutput, PublicKey},
    host_wrap::{load_asset_typed, store_asset_typed},
    tai::{Asset, AssetLockArgs, AssetTransferArgs, LockPrivilege, LockType},
};
use serde::{de::DeserializeOwned, Serialize};
use sha2::{Digest, Sha256};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

const MEMORY_SIZE: usize = 16384;

struct Memory {
    buf: [u8; MEMORY_SIZE],
    off: usize,
}

type ContractFunc = fn(AppContext, PackedValue) -> WasmResult<PackedValue>;

// Account struct used for testing.
#[derive(Default)]
struct Account {
    assets: HashMap<String, Vec<u8>>,
    data: HashMap<String, Vec<u8>>,
    contract: Vec<u8>,
}

struct ThreadData {
    memory: Memory,
    app_ctx: usize,
    accounts: HashMap<String, Account>,
    contract_methods: HashMap<String, ContractFunc>,
}

impl Default for ThreadData {
    fn default() -> Self {
        ThreadData {
            memory: Memory {
                buf: [0; MEMORY_SIZE],
                off: 0,
            },
            app_ctx: 0,
            accounts: HashMap::new(),
            contract_methods: HashMap::new(),
        }
    }
}

std::thread_local! {
    static THREADS_DATA: Rc<RefCell<ThreadData>> = Rc::new(RefCell::new(ThreadData::default()));
}

fn thread_data() -> Rc<RefCell<ThreadData>> {
    THREADS_DATA.with(|data| data.clone())
}

pub fn create_app_context<'a>(owner: &'a str, caller: &'a str) -> AppContext<'a> {
    AppContext {
        owner,
        caller,
        method: "",
        depth: 0,
        network: "skynet",
        origin: caller,
    }
}

pub fn get_app_ctx<'a>() -> &'a AppContext<'a> {
    let dat = thread_data();
    let addr = dat.borrow().app_ctx;
    unsafe { std::mem::transmute(addr) }
}

pub fn set_app_ctx<'a>(ctx: &'a AppContext<'a>) {
    let dat = thread_data();
    let prev_ctx = &mut dat.borrow_mut().app_ctx;
    *prev_ctx = unsafe { std::mem::transmute(ctx) };
}

fn get_account<'a>(accounts: &'a mut HashMap<String, Account>, id: &str) -> &'a mut Account {
    if !accounts.contains_key(id) {
        accounts.insert(id.to_owned(), Account::default());
    }
    accounts.get_mut(id).unwrap()
}

pub fn get_account_contract(account_id: &str) -> Vec<u8> {
    let dat = thread_data();
    let accounts = &mut dat.borrow_mut().accounts;
    let account = get_account(accounts, account_id);
    account.contract.clone()
}

pub fn is_callable(account_id: &str, method: &str) -> i32 {
    let dat = thread_data();
    let methods = &mut dat.borrow_mut().contract_methods;
    let key = format!("{}:{}", account_id, method);
    match methods.contains_key(&key) {
        true => 1,
        false => 0,
    }
}

pub fn set_account_contract(account_id: &str, contract: Vec<u8>) {
    let dat = thread_data();
    let accounts = &mut dat.borrow_mut().accounts;
    let account = get_account(accounts, account_id);
    account.contract = contract;
}

pub fn get_account_data(src_id: &str, key: &str) -> Vec<u8> {
    let dat = thread_data();
    let accounts = &mut dat.borrow_mut().accounts;
    let account = get_account(accounts, src_id);
    account.data.get(key).cloned().unwrap_or_default()
}

pub fn set_account_data(dst_id: &str, key: &str, data: &[u8]) {
    let dat = thread_data();
    let accounts = &mut dat.borrow_mut().accounts;
    let account = get_account(accounts, dst_id);
    match data.is_empty() {
        true => account.data.remove(key),
        false => account.data.insert(key.to_owned(), data.to_owned()),
    };
}

pub fn get_account_keys(src_id: &str) -> Vec<String> {
    let dat = thread_data();
    let accounts = &mut dat.borrow_mut().accounts;
    let account = get_account(accounts, src_id);
    account.data.keys().into_iter().cloned().collect()
}

pub fn get_account_asset(src_id: &str, asset: &str) -> Vec<u8> {
    let dat = thread_data();
    let accounts = &mut dat.borrow_mut().accounts;
    let account = get_account(accounts, src_id);
    account.assets.get(asset).cloned().unwrap_or_default()
}

pub fn set_account_asset(dst_id: &str, asset: &str, value: &[u8]) {
    let dat = thread_data();
    let accounts = &mut dat.borrow_mut().accounts;
    let account = get_account(accounts, dst_id);
    account.assets.insert(asset.to_owned(), value.to_owned());
}

pub fn get_account_asset_gen<T: DeserializeOwned + Default>(src_id: &str, asset_id: &str) -> T {
    let buf = get_account_asset(src_id, asset_id);
    rmp_deserialize(&buf).unwrap_or_default()
}

pub fn set_account_asset_gen<T: Serialize>(dst_id: &str, asset: &str, value: T) {
    let buf = rmp_serialize(&value).unwrap();
    set_account_asset(dst_id, asset, &buf);
}

/// Register a contract method to an account.
pub fn set_contract_method(account_id: &str, method: &str, func: ContractFunc) {
    let dat = thread_data();
    let methods = &mut dat.borrow_mut().contract_methods;
    let key = format!("{}:{}", account_id, method);
    methods.insert(key, func);
}

/// Register a contract hash to an account.
pub fn set_contract_hash(account_id: &str, contract: &[u8]) {
    let dat = thread_data();
    let accounts = &mut dat.borrow_mut().accounts;
    let account = get_account(accounts, account_id);
    account.contract = contract.to_vec();
}

pub fn memory_base() -> usize {
    thread_data().borrow().memory.buf.as_ptr() as usize
}

pub fn write_mem(buf: &[u8]) -> i32 {
    let dat = thread_data();
    let mut mem = &mut dat.as_ref().borrow_mut().memory;
    let prev_off = mem.off;
    let src = buf.as_ptr();
    let len = buf.len();
    mem.off += len;
    assert!(mem.off < MEMORY_SIZE, "Out of WASM (mocked) memory");
    unsafe {
        let dst = mem.buf.as_mut_ptr().add(prev_off);
        std::ptr::copy(src, dst, len);
    }
    prev_off as i32
}

pub fn call_wrap<F, T, U>(func: F, ctx: AppContext, args: T) -> WasmResult<U>
where
    F: FnOnce(AppContext, T) -> WasmResult<U>,
{
    set_app_ctx(&ctx);
    func(ctx, args)
}

#[no_mangle]
pub extern "C" fn hf_log(str_addr: i32, str_size: i32) {
    let msg = slice_from_mem(str_addr, str_size);
    println!("[HF] - {}", String::from_utf8_lossy(msg));
}

#[no_mangle]
pub extern "C" fn hf_emit(id_addr: i32, id_size: i32, data_addr: i32, data_size: i32) {
    let id = slice_from_mem(id_addr, id_size);
    let data = slice_from_mem(data_addr, data_size);
    println!(
        "[EMIT] - id: {}, data: {}",
        String::from_utf8_lossy(id),
        hex::encode(data)
    );
}

#[no_mangle]
pub extern "C" fn hf_get_keys(pattern_addr: i32, pattern_size: i32) -> WasmSlice {
    let ctx: &AppContext = get_app_ctx();
    let buf = slice_from_mem(pattern_addr, pattern_size);
    let pattern = unsafe { std::str::from_utf8_unchecked(buf) };

    let data_buf;

    let output = if pattern.is_empty() || &pattern[pattern.len() - 1..] != "*" {
        AppOutput {
            success: false,
            data: "last char of search pattern must be '*'".as_bytes(),
        }
    } else {
        let keys = get_account_keys(ctx.owner);
        let keys: Vec<String> = keys
            .iter()
            .cloned()
            .filter(|s| {
                (&pattern[..pattern.len() - 1]).is_empty()
                    || s.starts_with(&pattern[..pattern.len() - 1])
            })
            .collect();
        data_buf = rmp_serialize(&keys).unwrap_or_default();
        AppOutput {
            success: true,
            data: &data_buf,
        }
    };

    let buf = rmp_serialize(&output).unwrap();
    slice_to_wslice(&buf)
}

#[no_mangle]
pub extern "C" fn hf_store_data(key_addr: i32, key_size: i32, data_addr: i32, data_size: i32) {
    let ctx: &AppContext = get_app_ctx();
    let buf = slice_from_mem(key_addr, key_size);
    let key = unsafe { std::str::from_utf8_unchecked(buf) };
    let data = slice_from_mem(data_addr, data_size);
    set_account_data(ctx.owner, key, data);
}

#[no_mangle]
pub extern "C" fn hf_load_data(key_addr: i32, key_size: i32) -> WasmSlice {
    let ctx: &AppContext = get_app_ctx();
    let buf = slice_from_mem(key_addr, key_size);
    let key = unsafe { std::str::from_utf8_unchecked(buf) };
    let buf = get_account_data(ctx.owner, key);
    slice_to_wslice(&buf)
}

#[no_mangle]
pub extern "C" fn hf_get_account_contract(id_addr: i32, id_size: i32) -> WasmSlice {
    let buf = slice_from_mem(id_addr, id_size);
    let account_id = unsafe { std::str::from_utf8_unchecked(buf) };
    let buf = get_account_contract(account_id);
    slice_to_wslice(&buf)
}

#[no_mangle]
pub extern "C" fn hf_is_callable(
    id_addr: i32,
    id_size: i32,
    method_addr: i32,
    method_size: i32,
) -> i32 {
    let buf = slice_from_mem(id_addr, id_size);
    let account_id = unsafe { std::str::from_utf8_unchecked(buf) };
    let buf = slice_from_mem(method_addr, method_size);
    let method = unsafe { std::str::from_utf8_unchecked(buf) };
    is_callable(account_id, method)
}

#[no_mangle]
pub extern "C" fn hf_remove_data(key_addr: i32, key_size: i32) {
    let ctx: &AppContext = get_app_ctx();
    let buf = slice_from_mem(key_addr, key_size);
    let key = unsafe { std::str::from_utf8_unchecked(buf) };
    set_account_data(ctx.owner, key, &[]);
}

#[no_mangle]
pub extern "C" fn hf_load_asset(src_id_addr: i32, src_id_size: i32) -> WasmSlice {
    let ctx: &AppContext = get_app_ctx();
    let buf = slice_from_mem(src_id_addr, src_id_size);
    let src_id = unsafe { std::str::from_utf8_unchecked(buf) };
    let buf = get_account_asset(src_id, ctx.owner);
    slice_to_wslice(&buf)
}

#[no_mangle]
pub extern "C" fn hf_store_asset(
    dst_id_addr: i32,
    dst_id_size: i32,
    value_addr: i32,
    value_size: i32,
) {
    let ctx: &AppContext = get_app_ctx();
    let buf = slice_from_mem(dst_id_addr, dst_id_size);
    let dst_id = unsafe { std::str::from_utf8_unchecked(buf) };
    let value = slice_from_mem(value_addr, value_size);
    set_account_asset(dst_id, ctx.owner, value);
}

#[no_mangle]
pub extern "C" fn hf_sha256(data_addr: i32, data_size: i32) -> WasmSlice {
    let data = slice_from_mem(data_addr, data_size);

    let mut hasher = Sha256::new();
    hasher.update(data);
    let digest = hasher.finalize();

    slice_to_wslice(digest.as_ref())
}

#[no_mangle]
pub extern "C" fn hf_drand(max: u64) -> u64 {
    max / 2
}

// Use the first byte of the sign to return success or error.
#[no_mangle]
pub extern "C" fn hf_verify(
    pk_addr: i32,
    pk_size: i32,
    data_addr: i32,
    data_size: i32,
    sign_addr: i32,
    sign_size: i32,
) -> i32 {
    let pk = slice_from_mem(pk_addr, pk_size);
    let _pk: PublicKey = match rmp_deserialize(pk) {
        Ok(val) => val,
        Err(_) => return 0,
    };
    let _data = slice_from_mem(data_addr, data_size);
    let sign = slice_from_mem(sign_addr, sign_size);

    sign[0] as i32
}

#[no_mangle]
pub extern "C" fn hf_call(
    account_addr: i32,
    account_size: i32,
    method_addr: i32,
    method_size: i32,
    data_addr: i32,
    data_size: i32,
) -> WasmSlice {
    let buf = Vec::<u8>::new();
    let contract_addr = slice_to_mem(&buf);
    hf_s_call(
        account_addr,
        account_size,
        contract_addr,
        0,
        method_addr,
        method_size,
        data_addr,
        data_size,
    )
}

#[no_mangle]
pub extern "C" fn hf_s_call(
    account_addr: i32,
    account_size: i32,
    contract_addr: i32,
    contract_size: i32,
    method_addr: i32,
    method_size: i32,
    data_addr: i32,
    data_size: i32,
) -> WasmSlice {
    let ctx: &AppContext = get_app_ctx();
    let slice = slice_from_mem(account_addr, account_size);
    let account = unsafe { std::str::from_utf8_unchecked(slice) };
    let contract = slice_from_mem(contract_addr, contract_size).to_owned();
    let slice = slice_from_mem(method_addr, method_size);
    let method = unsafe { std::str::from_utf8_unchecked(slice) };
    let args = slice_from_mem(data_addr, data_size).to_owned();

    println!(
        "[s_call] - {}::{}::{}({})",
        account,
        hex::encode(contract.clone()),
        method,
        hex::encode(args.clone())
    );

    let method_func = {
        let method_name = format!("{}:{}", account, method);
        let dat = thread_data();

        if !contract.is_empty() {
            let val = match &dat.borrow().accounts.get(account) {
                Some(acc) => acc.contract == contract,

                None => false, // return AppOutput::ko("incompatible contract app").into(),
            };
            if !val {
                return AppOutput::ko("incompatible contract app").into();
            }
        }

        let method_func = {
            let map = &dat.borrow().contract_methods;
            map.get(&method_name).copied()
        };
        match method_func {
            Some(method) => method.to_owned(),
            None => return AppOutput::ko("method not found").into(),
        }
    };

    let prev_ctx = get_app_ctx();

    let ctx = AppContext {
        owner: account,
        caller: ctx.owner,
        method,
        depth: ctx.depth + 1,
        network: ctx.network,
        origin: ctx.origin,
    };

    set_app_ctx(&ctx);
    let result = match method_func(ctx, PackedValue(args)) {
        Ok(res) => AppOutput::ok(res.as_ref()).into(),
        Err(err) => AppOutput::ko(&err.to_string()).into(),
    };
    set_app_ctx(prev_ctx);

    result
}

/// Mocked TAI Asset `transfer` method used by the tests.
pub fn asset_transfer(_ctx: AppContext, args: PackedValue) -> WasmResult<PackedValue> {
    let args: AssetTransferArgs = rmp_deserialize(&args).unwrap();

    // Withdraw
    let mut value: Asset = load_asset_typed(args.from);
    if value.lock.is_some() {
        return Err(WasmError::new("source account locked"));
    }
    if value.units < args.units {
        return Err(WasmError::new("error during transfer"));
    }
    value.units -= args.units;
    store_asset_typed(args.from, value);

    // Deposit
    let mut value: Asset = load_asset_typed(args.to);
    if value.lock.is_some() {
        return Err(WasmError::new("destination account locked"));
    }
    value.units += args.units;
    store_asset_typed(args.to, value);

    let buf = rmp_serialize(&()).unwrap();
    Ok(PackedValue(buf))
}

/// Mocked TAI Asset `balance` method used by the tests.
pub fn asset_balance(ctx: AppContext, _args: PackedValue) -> WasmResult<PackedValue> {
    let value: Asset = load_asset_typed(ctx.caller);
    if value.lock.is_some() {
        return Err(WasmError::new("account locked"));
    }
    let buf = rmp_serialize(&value.units).unwrap();
    Ok(PackedValue(buf))
}

/// Mocked Advanced Asset `transfer` method used by the tests.
pub fn adv_asset_transfer(_ctx: AppContext, args: PackedValue) -> WasmResult<PackedValue> {
    let args: AssetTransferArgs = rmp_deserialize(&args).unwrap();

    // Withdraw
    let mut value_units: u64 = load_asset_typed(args.from);
    if value_units < args.units {
        return Err(WasmError::new("error during transfer"));
    }
    value_units -= args.units;
    store_asset_typed(args.from, value_units);

    // Deposit
    let mut value_units: u64 = load_asset_typed(args.to);
    value_units += args.units;
    store_asset_typed(args.to, value_units);

    let buf = rmp_serialize(&()).unwrap();
    Ok(PackedValue(buf))
}

/// Mocked TAI Asset `balance` method used by the tests.
pub fn adv_asset_balance(ctx: AppContext, _args: PackedValue) -> WasmResult<PackedValue> {
    let value: u64 = load_asset_typed(ctx.caller);
    let buf = rmp_serialize(&value).unwrap();
    Ok(PackedValue(buf))
}

/// Mocked TAI Asset `lock` method used by the tests.
pub fn asset_lock(ctx: AppContext, args: PackedValue) -> WasmResult<PackedValue> {
    let args: AssetLockArgs = rmp_deserialize(&args).unwrap();
    let mut value: Asset = load_asset_typed(ctx.caller);
    let prev_lock = value.lock;
    value.lock = match args.lock {
        LockType::None => None,
        lock_type => Some((LockPrivilege::Owner, lock_type)),
    };
    store_asset_typed(ctx.caller, value);
    let buf = rmp_serialize(&prev_lock).unwrap();
    Ok(PackedValue(buf))
}
