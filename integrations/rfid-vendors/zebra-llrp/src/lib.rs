//! eel-rfid-zebra-llrp: Zebra FX9600 vendor adapter via LLRP (open standard).
//!
//! - `adapter::ZebraLlrpHandler` — HTTP client wrapping the py/rfid-reader-driver
//!   endpoint that uses the LLRP-based zebra_llrp adapter.  No commercial SDK.
//! - `double::MockZebraLlrp` — in-memory double with tamper injection.

pub mod adapter;
pub mod double;

pub use adapter::ZebraLlrpHandler;
pub use double::MockZebraLlrp;
