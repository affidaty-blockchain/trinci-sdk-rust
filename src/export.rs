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

//! Wasm module entry point definition

use crate::{
    common::*,
    core::{AppInput, AppOutput},
};
use std::{alloc::Layout, mem::align_of};

/// Memory allocation in the wasm linear memory from the host.
///
/// This allocate a buffer on the wasm memory
#[no_mangle]
extern "C" fn alloc(len: usize) -> *mut u8 {
    unsafe { std::alloc::alloc(Layout::from_size_align_unchecked(len, align_of::<usize>())) }
}

extern "Rust" {
    #[doc(hidden)]
    fn app_run(ctx: AppContext, args: &[u8]) -> Result<Vec<u8>, WasmError>;
}

impl<'a> AppOutput<'a> {
    pub(crate) fn ok(data: &'a [u8]) -> Self {
        AppOutput {
            success: true,
            data,
        }
    }

    pub(crate) fn ko(msg: &'a str) -> Self {
        AppOutput {
            success: false,
            data: msg.as_bytes(),
        }
    }
}

impl From<AppOutput<'_>> for WasmSlice {
    fn from(app_res: AppOutput) -> Self {
        let buf = rmp_serialize(&app_res).unwrap_or_default();
        slice_to_wslice(buf.leak())
    }
}

/// Smart contracts main entry point.
///
/// When C structure is returned by value, then its return address is expected to
/// be passed as a first function parameter!
#[no_mangle]
extern "C" fn run(ctx_addr: i32, ctx_size: i32, args_addr: i32, args_size: i32) -> WasmSlice {
    let slice = slice_from_mem(ctx_addr, ctx_size);
    let ctx: AppInput = match rmp_deserialize(slice) {
        Ok(value) => value,
        Err(_err) => return AppOutput::ko("malformed input").into(),
    };

    let slice = slice_from_mem(args_addr, args_size);

    let res = unsafe { app_run(ctx, slice) };

    match res {
        Ok(buf) => AppOutput::ok(&buf).into(),
        Err(err) => AppOutput::ko(&err.to_string()).into(),
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::{value, Value};

    #[no_mangle]
    fn app_run(ctx: &AppContext, args: &[u8]) -> Result<Vec<u8>, WasmError> {
        match ctx.method {
            "foo" => {
                let input: Value = rmp_deserialize(args)?;
                let age: u64 = get_value_as_u64!(input, "age").unwrap();
                let output = value!(age + 1);
                rmp_serialize(&output)
            }
            "bar" => Err(WasmError::new("bad args")),
            _ => Err(WasmError::new("bad method")),
        }
    }

    const CALLER: &str = "QmYHnEQLdf5h7KYbjFPuHSRk2SPgdXrJWFh5W696HPfq7i";

    fn run_wrapper(method: &str, args: Value) -> std::result::Result<Value, String> {
        let input = AppInput {
            caller: CALLER,
            owner: CALLER,
            method,
            depth: 0,
            network: "skynet",
            origin: CALLER,
        };
        let input_buf = rmp_serde::to_vec(&input).unwrap();
        let input_addr = slice_to_mem(&input_buf);

        let args = rmp_serde::to_vec_named(&args).unwrap();
        let args_addr = slice_to_mem(&args);

        let wslice = run(
            input_addr,
            input_buf.len() as i32,
            args_addr,
            args.len() as i32,
        );

        let slice = slice_from_wslice(wslice);
        let res: AppOutput = rmp_serde::from_read_ref(slice).unwrap();

        match res.success {
            true => match rmp_deserialize(res.data) {
                Ok(value) => Ok(value),
                Err(err) => Err(err.to_string()),
            },
            false => Err(String::from_utf8_lossy(res.data).to_string()),
        }
    }

    #[test]
    fn run_method_with_success() {
        let method = "foo";
        let args = value!({
            "name": "Cole",
            "age": 33
        });

        let res = run_wrapper(method, args);

        let value = match res {
            Ok(value) => value,
            Err(_) => panic!("Unexpected failure result"),
        };
        assert_eq!(value, 34);
    }

    #[test]
    fn run_method_with_bad_args() {
        let method = "bar";
        let args = value!(null);

        let res = run_wrapper(method, args);

        let msg = match res {
            Ok(_) => panic!("Unexpected success result"),
            Err(str) => str,
        };
        assert_eq!(msg, "bad args");
    }

    #[test]
    fn run_bad_utf8_method() {
        let buf = vec![240, 159, 146];
        let method = unsafe { std::str::from_utf8_unchecked(&buf) };
        let args = value!(null);

        let res = run_wrapper(method, args);

        let msg = match res {
            Ok(_) => panic!("Unexpected success result"),
            Err(str) => str,
        };
        assert_eq!(msg, "malformed input");
    }

    // {
    //   true,
    //   82a46e616d65a4436f6c65a361676521
    // }
    #[test]
    fn serialize_app_result_success() {
        let returns = value!({
            "age": 33,
            "name": "Cole",
        });
        let buf = rmp_serde::to_vec_named(&returns).unwrap();
        let app_res = AppOutput::ok(&buf);

        let buf = rmp_serde::to_vec(&app_res).unwrap();

        assert_eq!(hex::encode(buf), "92c3c41082a361676521a46e616d65a4436f6c65");
    }

    // {
    //   false,
    //   6261642061726773  (bad args string)
    // }
    #[test]
    fn serialize_app_result_fail() {
        let app_res = AppOutput::ko("bad args");

        let buf = rmp_serde::to_vec(&app_res).unwrap();

        assert_eq!(hex::encode(buf), "92c2c4086261642061726773");
    }
}
