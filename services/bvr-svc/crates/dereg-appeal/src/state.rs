use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    /// Filed appeals, their decisions, and the USSD session log.
    ///
    /// The pool alone was the whole of this state, so the handlers built their
    /// answers:  recorded a reinstatement against any appeal
    /// id and  returned an empty page.
    pub store: crate::store::SharedDeregStore,
}

impl AppState {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            store: crate::store::DeregStore::new(),
        }
    }
}
