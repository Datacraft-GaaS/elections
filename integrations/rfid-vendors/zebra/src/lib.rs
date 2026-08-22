//! eel-rfid-zebra: Zebra FX9600 vendor adapter.
//!
//! - `adapter::ZebraHandler` — HTTP client wrapping the py/rfid-reader-driver endpoint.
//! - `double::MockZebra` — in-memory double with tamper injection.

pub mod adapter;
pub mod double;

pub use adapter::ZebraHandler;
pub use double::MockZebra;
