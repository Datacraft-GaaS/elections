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

//! eel-rfid-cipherlab-ble: CipherLab RS36 handheld adapter via BLE.
//!
//! - `adapter::CipherLabBleHandler` — HTTP client wrapping the py/rfid-reader-driver
//!   endpoint that uses the bleak-based BLE adapter.  No commercial CipherLab SDK.
//! - `double::MockCipherLabBle` — in-memory double with tamper + battery-low injection.
#![warn(missing_docs)]

pub mod adapter;
pub mod double;

pub use adapter::CipherLabBleHandler;
pub use double::MockCipherLabBle;
