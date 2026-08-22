//! eel-rfid-cipherlab-ble: CipherLab RS36 handheld adapter via BLE.
//!
//! - `adapter::CipherLabBleHandler` — HTTP client wrapping the py/rfid-reader-driver
//!   endpoint that uses the bleak-based BLE adapter.  No commercial CipherLab SDK.
//! - `double::MockCipherLabBle` — in-memory double with tamper + battery-low injection.

pub mod adapter;
pub mod double;

pub use adapter::CipherLabBleHandler;
pub use double::MockCipherLabBle;
