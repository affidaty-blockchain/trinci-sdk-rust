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

//! Serde Value rappresentation
//! 
//! TODO: these tests shall be in the serde-value crate.

#[cfg(test)]
mod value_serialize_tests {
    use crate::common::rmp_serialize;
    use serde_value::Value;

    #[test]
    fn unit_serialize() {
        let val = Value::Unit;

        let buf = rmp_serialize(&val).unwrap();

        assert_eq!("c0", hex::encode(&buf));
    }

    #[test]
    fn bool_serialize() {
        let val = Value::Bool(true);

        let buf = rmp_serialize(&val).unwrap();

        assert_eq!("c3", hex::encode(&buf));
    }

    #[test]
    fn u32_serialize() {
        let val = Value::U32(42u32);

        let buf = rmp_serialize(&val).unwrap();

        assert_eq!("2a", hex::encode(&buf));
    }

    #[test]
    fn i32_serialize() {
        let val = Value::I32(i32::MAX);

        let buf = rmp_serialize(&val).unwrap();

        assert_eq!("ce7fffffff", hex::encode(&buf));
    }

    #[test]
    fn u64_serialize() {
        let val = Value::U64(9000000000000000000u64);

        let buf = rmp_serialize(&val).unwrap();

        assert_eq!("cf7ce66c50e2840000", hex::encode(&buf));
    }

    #[test]
    fn i64_serialize() {
        let val = Value::I64(9000000000000000000);

        let buf = rmp_serialize(&val).unwrap();

        assert_eq!("cf7ce66c50e2840000", hex::encode(&buf));
    }

    #[test]
    fn f32_serialize() {
        let val = Value::F32(42.0f32);

        let buf = rmp_serialize(&val).unwrap();

        assert_eq!("ca42280000", hex::encode(&buf));
    }

    #[test]
    fn f64_serialize() {
        let val = Value::F64(42.0);

        let buf = rmp_serialize(&val).unwrap();

        assert_eq!("cb4045000000000000", hex::encode(&buf));
    }

    #[test]
    fn string_serialize() {
        let val = Value::String("Hello".to_string());

        let buf = rmp_serialize(&val).unwrap();

        assert_eq!("a548656c6c6f", hex::encode(&buf));
    }

    #[test]
    fn bytes_serialize() {
        let val = Value::Bytes(vec![1, 2, 3]);

        let buf = rmp_serialize(&val).unwrap();

        assert_eq!("c403010203", hex::encode(&buf));
    }

    #[test]
    fn seq_serialize() {
        let val = Value::Seq(vec![Value::I32(1), Value::I32(2), Value::I32(3)]);

        let buf = rmp_serialize(&val).unwrap();

        assert_eq!("93010203", hex::encode(&buf));
    }

    #[test]
    fn map_serialize() {
        let val = Value::Map(
            vec![
                ("k1".into(), Value::Bytes(vec![1, 2, 3])),
                ("k2".into(), Value::Bool(true)),
                ("k3".into(), Value::I32(3)),
            ]
            .into_iter()
            .collect(),
        );

        let buf = rmp_serialize(&val).unwrap();

        assert_eq!("83a26b31c403010203a26b32c3a26b3303", hex::encode(&buf));
    }

    #[test]
    fn complex_serialization() {
        let val = Value::Map(
            vec![
                (
                    "k0".into(),
                    Value::Seq(vec![
                        Value::Map(vec![("x".into(), Value::Bool(false))].into_iter().collect()),
                        Value::I32(2),
                        Value::F32(3.2),
                    ]),
                ),
                ("k1".into(), Value::Bytes(vec![1, 2, 3])),
                ("k2".into(), Value::Bool(true)),
                ("k3".into(), Value::I32(3)),
                (
                    "k4".into(),
                    Value::Map(
                        vec![
                            ("a".into(), Value::Unit),
                            ("b".into(), Value::F64(42.3f64)),
                            ("c".into(), Value::Bool(false)),
                        ]
                        .into_iter()
                        .collect(),
                    ),
                ),
            ]
            .into_iter()
            .collect(),
        );

        let buf = rmp_serialize(&val).unwrap();

        assert_eq!(
            "85a26b309381a178c202ca404ccccda26b31c403010203a26b32c3a26b3303a26b3483a161c0a162cb4045266666666666a163c2",
            hex::encode(&buf)
        );
    }
}

#[cfg(test)]
mod value_deserialize_tests {
    use crate::common::rmp_deserialize;
    use serde_value::Value;

    #[test]
    fn unit_deserialize() {
        let val = Value::Unit;
        let buf = hex::decode("c0").unwrap();

        let val_des: Value = rmp_deserialize(&buf).unwrap();

        assert_eq!(val, val_des);
    }

    #[test]
    fn bool_deserialize() {
        let val = Value::Bool(false);
        let buf = hex::decode("c2").unwrap();

        let val_des: Value = rmp_deserialize(&buf).unwrap();

        assert_eq!(val, val_des);
    }

    #[test]
    fn u32_deserialize() {
        let val = Value::U32(4000000000u32);
        let buf = hex::decode("CEEE6B2800").unwrap();

        let val_des: Value = rmp_deserialize(&buf).unwrap();

        assert_eq!(val, val_des);
    }

    #[test]
    fn u64_deserialize() {
        let val = Value::U64(18000000000000000000u64);
        let buf = hex::decode("CFF9CCD8A1C5080000").unwrap();

        let val_des: Value = rmp_deserialize(&buf).unwrap();

        assert_eq!(val, val_des);
    }

    #[test]
    fn i32_deserialize() {
        let buf = hex::decode("ce7fffffff").unwrap();

        let val_des: Value = rmp_deserialize(&buf).unwrap();

        assert_eq!(i32::MAX, val_des);
    }

    #[test]
    fn i64_deserialize() {
        let val = Value::I64(-9000000000000000000i64);
        let buf = hex::decode("D3831993AF1D7C0000").unwrap();

        let val_des: Value = rmp_deserialize(&buf).unwrap();

        assert_eq!(val, val_des);
    }

    #[test]
    fn f32_deserialize() {
        let val = Value::F32(42.0f32);
        let buf = hex::decode("ca42280000").unwrap();

        let val_des: Value = rmp_deserialize(&buf).unwrap();

        assert_eq!(val, val_des);
    }

    #[test]
    fn f64_deserialize() {
        let val = Value::F64(42.0f64);
        let buf = hex::decode("cb4045000000000000").unwrap();

        let val_des: Value = rmp_deserialize(&buf).unwrap();

        assert_eq!(val, val_des);
    }

    #[test]
    fn string_deserialize() {
        let val = Value::String("Hello".to_string());
        let buf = hex::decode("a548656c6c6f").unwrap();

        let val_des: Value = rmp_deserialize(&buf).unwrap();

        assert_eq!(val, val_des);
    }

    #[test]
    fn bytes_deserialize() {
        let val = Value::Bytes(vec![1, 2, 3]);
        let buf = hex::decode("c403010203").unwrap();

        let val_des: Value = rmp_deserialize(&buf).unwrap();

        assert_eq!(val, val_des);
    }

    #[test]
    fn array_deserialize() {
        let val = Value::Seq(vec![Value::U8(1), Value::U8(2), Value::U8(3)]);
        let buf = hex::decode("93010203").unwrap();

        let val_des: Value = rmp_deserialize(&buf).unwrap();

        assert_eq!(val, val_des);
    }

    #[test]
    fn map_deserialize() {
        let val = Value::Map(
            vec![("1".into(), Value::U8(42)), ("2".into(), Value::U8(43))]
                .into_iter()
                .collect(),
        );
        let buf = hex::decode("82a1312aa1322b").unwrap();

        let val_des: Value = rmp_deserialize(&buf).unwrap();

        assert_eq!(val, val_des);
    }

    #[test]
    fn complex_deserialization() {
        let val = Value::Map(
            vec![
                (
                    "k0".into(),
                    Value::Seq(vec![
                        Value::Map(vec![("x".into(), Value::Bool(false))].into_iter().collect()),
                        Value::U8(2),
                        Value::F32(3.2),
                    ]),
                ),
                ("k1".into(), Value::Bytes(vec![1, 2, 3])),
                ("k2".into(), Value::Bool(true)),
                ("k3".into(), Value::U8(3)),
                (
                    "k4".into(),
                    Value::Map(
                        vec![
                            ("a".into(), Value::Unit),
                            ("b".into(), Value::F64(42.3f64)),
                            ("c".into(), Value::Bool(false)),
                        ]
                        .into_iter()
                        .collect(),
                    ),
                ),
            ]
            .into_iter()
            .collect(),
        );
        let buf = hex::decode(
            "85a26b309381a178c202ca404ccccda26b31c403010203a26b32c3a26b3303a26b3483a161c0a162cb4045266666666666a163c2",
        )
        .unwrap();

        let val_des: Value = rmp_deserialize(&buf).unwrap();

        assert_eq!(val, val_des);
    }
}
