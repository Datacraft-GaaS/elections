use sqlx::PgPool;

use crate::store::Store;

#[derive(Clone)]
pub struct AppState {
    pub store: Store,
}

impl AppState {
    pub fn new(pool: PgPool) -> Self {
        Self { store: Store::pg(pool.clone()) }
    }
    pub fn new_mem() -> Self {
        Self { store: Store::mem() }
    }
    /// Pool accessor for legacy routes that do direct sqlx (batches, blacklist, status).
    /// Panics in `Mem` mode — those routes are not used in hermetic tests.
    pub fn pool(&self) -> &PgPool {
        match &self.store {
            Store::Pg(p) => p,
            Store::Mem(_) => panic!("pool not available in Mem store — use Store::Pg in prod"),
        }
    }
}
