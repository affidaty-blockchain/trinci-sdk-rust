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

//! Trinci Applications Interface (TAI).

use crate::PackedValue;
use serde::{Deserialize, Serialize};

/// Asset's Lock Level.
/// Authority level order: Owner < Contract < Creator.
#[derive(Serialize, Deserialize, Debug, PartialEq, Copy, Clone)]
pub enum LockPrivilege {
    /// Set by the owner via direct asset invocation.
    Owner,
    /// Set by the owner via smart contract.
    Contract,
    /// Set by the asset's creator.
    Creator,
}

impl Default for LockPrivilege {
    fn default() -> Self {
        LockPrivilege::Owner
    }
}

/// Asset's Lock Type.
/// Asset lock for inflow, outflow, both, none.
#[derive(Serialize, Deserialize, Debug, PartialEq, Copy, Clone)]
pub enum LockType {
    /// Unlocks the asset flow both way from and to the account.
    None = 0,
    /// Locks the asset flow to the account.
    Deposit = 1,
    /// Locks the asset flow from the account.
    Withdraw = 2,
    /// Locks the asset flow both way from and to the account.
    Full = 3,
}

impl Default for LockType {
    fn default() -> Self {
        LockType::None
    }
}

/// Standard asset descriptor that can be locked.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
pub struct Asset {
    // Number of asset units.
    pub units: u64,
    // Lock level.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lock: Option<(LockPrivilege, LockType)>,
}

impl Asset {
    pub fn new(val: u64) -> Self {
        Asset {
            units: val,
            lock: None,
        }
    }
}

/// Arguments for asset `lock` method.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
pub struct AssetLockArgs<'a> {
    pub to: &'a str,
    pub lock: LockType,
}

/// Arguments for the asset `transfer` method.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
pub struct AssetTransferArgs<'a> {
    pub from: &'a str,
    pub to: &'a str,
    pub units: u64,
    #[serde(with = "serde_bytes")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<Vec<u8>>,
}

/// Arguments for the asset `balance` method.
pub type AssetBalanceArgs = PackedValue;

/// Returns for the asset `balance` method.
pub type AssetBalanceRets = u64;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{rmp_deserialize, rmp_serialize};

    const ASSET_NO_LOCK_HEX: &str = "9164";
    const ASSET_FULL_LOCK_HEX: &str = "926492a743726561746f72a446756c6c";
    const ASSET_DEPOSIT_LOCK_HEX: &str = "926492a743726561746f72a74465706f736974";
    const ASSET_WITHDRAW_LOCK_HEX: &str = "926492a743726561746f72a85769746864726177";
    const ASSET_TRANSFER_ARGS_HEX: &str = "93d92e516d54654e5063516e6f78696e6239626351687546787465545134734e337153574a4e6f486a67457238347a4e59d92e516d5a4b72666f71385a746b483434353337337146516f386d4a554563316a783161764d4c59394a52544d4a4d447b";

    fn create_test_transfer_args() -> AssetTransferArgs<'static> {
        AssetTransferArgs {
            from: "QmTeNPcQnoxinb9bcQhuFxteTQ4sN3qSWJNoHjgEr84zNY",
            to: "QmZKrfoq8ZtkH445373qFQo8mJUEc1jx1avMLY9JRTMJMD",
            units: 123,
            data: None,
        }
    }

    #[test]
    fn asset_no_lock_serialize() {
        let asset = Asset {
            units: 100,
            lock: None,
        };

        let buf = rmp_serialize(&asset).unwrap();

        assert_eq!(buf, hex::decode(ASSET_NO_LOCK_HEX).unwrap());
    }

    #[test]
    fn asset_no_lock_deserialize() {
        let buf = hex::decode(ASSET_NO_LOCK_HEX).unwrap();

        let asset: Asset = rmp_deserialize(&buf).unwrap();

        assert_eq!(asset.units, 100);
        assert_eq!(asset.lock, None);
    }

    #[test]
    fn asset_full_lock_serialize() {
        let asset = Asset {
            units: 100,
            lock: Some((LockPrivilege::Creator, LockType::Full)),
        };

        let buf = rmp_serialize(&asset).unwrap();

        assert_eq!(hex::encode(&buf), ASSET_FULL_LOCK_HEX);
    }

    #[test]
    fn asset_full_lock_deserialize() {
        let buf = hex::decode(ASSET_FULL_LOCK_HEX).unwrap();

        let asset: Asset = rmp_deserialize(&buf).unwrap();

        assert_eq!(asset.units, 100);
        assert_eq!(asset.lock, Some((LockPrivilege::Creator, LockType::Full)));
    }

    #[test]
    fn asset_deposit_lock_serialize() {
        let asset = Asset {
            units: 100,
            lock: Some((LockPrivilege::Creator, LockType::Deposit)),
        };

        let buf = rmp_serialize(&asset).unwrap();

        assert_eq!(hex::encode(&buf), ASSET_DEPOSIT_LOCK_HEX);
    }

    #[test]
    fn asset_deposit_lock_deserialize() {
        let buf = hex::decode(ASSET_DEPOSIT_LOCK_HEX).unwrap();

        let asset: Asset = rmp_deserialize(&buf).unwrap();

        assert_eq!(asset.units, 100);
        assert_eq!(
            asset.lock,
            Some((LockPrivilege::Creator, LockType::Deposit))
        );
    }

    #[test]
    fn asset_withdraw_lock_serialize() {
        let asset = Asset {
            units: 100,
            lock: Some((LockPrivilege::Creator, LockType::Withdraw)),
        };

        let buf = rmp_serialize(&asset).unwrap();

        assert_eq!(hex::encode(&buf), ASSET_WITHDRAW_LOCK_HEX);
    }

    #[test]
    fn asset_withdraw_lock_deserialize() {
        let buf = hex::decode(ASSET_WITHDRAW_LOCK_HEX).unwrap();

        let asset: Asset = rmp_deserialize(&buf).unwrap();

        assert_eq!(asset.units, 100);
        assert_eq!(
            asset.lock,
            Some((LockPrivilege::Creator, LockType::Withdraw))
        );
    }

    #[test]
    fn asset_transfer_args_serialize() {
        let args = create_test_transfer_args();

        let buf = rmp_serialize(&args).unwrap();

        assert_eq!(hex::encode(&buf), ASSET_TRANSFER_ARGS_HEX);
    }

    #[test]
    fn asset_transfer_args_deserialize() {
        let expected = create_test_transfer_args();
        let buf = hex::decode(ASSET_TRANSFER_ARGS_HEX).unwrap();

        let args: AssetTransferArgs = rmp_deserialize(&buf).unwrap();

        assert_eq!(args, expected);
    }
}
