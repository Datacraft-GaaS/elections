//! Application state shared across handlers.

pub struct AppState {
    pub mock_mode: bool,
}

impl AppState {
    pub fn new() -> Self {
        Self { mock_mode: true }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
