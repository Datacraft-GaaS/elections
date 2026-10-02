// Copyright (c) 2026 Nyimbi Odero
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Serde helpers for fixed-size byte arrays larger than 32 bytes.
//!
//! Serde's blanket impls only cover `[T; 0..=32]` in this workspace's toolchain.
//! Use `#[serde(with = "crate::serde_array")]` on `[u8; N]` fields where N > 32.

use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub fn serialize<S, const N: usize>(bytes: &[u8; N], ser: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    bytes.as_slice().serialize(ser)
}

pub fn deserialize<'de, D, const N: usize>(de: D) -> Result<[u8; N], D::Error>
where
    D: Deserializer<'de>,
{
    let v: Vec<u8> = Vec::deserialize(de)?;
    v.try_into().map_err(|v: Vec<u8>| {
        serde::de::Error::custom(format!("expected {} bytes, got {}", N, v.len()))
    })
}
