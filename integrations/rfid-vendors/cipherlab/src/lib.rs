//! eel-rfid-cipherlab: CipherLab RS36 handheld vendor adapter.
//!
//! - `adapter::CipherLabHandler` — HTTP client over py/rfid-reader-driver with circuit breaker.
//! - `double::MockCipherLab` — in-memory double with tamper injection for handheld PDA chaos.
#![warn(missing_docs)]

pub mod adapter;
pub mod double;

pub use adapter::CipherLabHandler;
pub use double::MockCipherLab;
