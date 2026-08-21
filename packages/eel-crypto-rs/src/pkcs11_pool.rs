//! PKCS#11 session pool over the `cryptoki` crate.
//!
//! Maintains a pool of open PKCS#11 sessions to avoid the overhead of
//! opening a new session per operation.  Key roles are defined by
//! [`crate::KeyRole`].
//!
//! # Feature
//!
//! Requires the `pkcs11` feature and a PKCS#11 runtime library at link time
//! (e.g. SoftHSMv2).
//!
//! # Example
//!
//! ```no_run
//! use eel_crypto_rs::pkcs11_pool::{PkcsPool, PkcsPoolConfig};
//! use std::path::Path;
//! let cfg = PkcsPoolConfig {
//!     module_path: "/usr/lib/softhsm/libsofthsm2.so".into(),
//!     slot_id: 0,
//!     pin: "1234".into(),
//!     max_sessions: 4,
//! };
//! let pool = PkcsPool::open(cfg).expect("open PKCS#11 pool");
//! ```

use std::path::PathBuf;
use std::sync::Arc;

use cryptoki::context::{CInitializeArgs, Pkcs11};
use cryptoki::session::Session;
use cryptoki::slot::Slot;
use parking_lot::Mutex;

use crate::{CryptoError, KeyRole};

/// Configuration for [`PkcsPool`].
#[derive(Debug, Clone)]
pub struct PkcsPoolConfig {
    /// Filesystem path to the PKCS#11 shared library.
    pub module_path: PathBuf,
    /// Slot index to open sessions on.
    pub slot_id: u64,
    /// SO or user PIN.
    pub pin: String,
    /// Maximum number of sessions to keep open.
    pub max_sessions: usize,
}

/// A pool of open PKCS#11 sessions.
///
/// Cloning the pool shares the same underlying session pool.
///
/// # Example
///
/// ```no_run
/// use eel_crypto_rs::pkcs11_pool::{PkcsPool, PkcsPoolConfig};
/// let pool = PkcsPool::open(PkcsPoolConfig {
///     module_path: "/usr/lib/softhsm/libsofthsm2.so".into(),
///     slot_id: 0,
///     pin: "1234".into(),
///     max_sessions: 2,
/// })
/// .unwrap();
/// ```
#[derive(Clone)]
pub struct PkcsPool(Arc<Inner>);

struct Inner {
    ctx: Pkcs11,
    slot: Slot,
    pin: String,
    available: Mutex<Vec<Session>>,
    max_sessions: usize,
}

impl PkcsPool {
    /// Open a PKCS#11 session pool.
    ///
    /// Initialises the PKCS#11 library and opens `config.max_sessions` RW
    /// sessions on the configured slot.
    pub fn open(config: PkcsPoolConfig) -> Result<Self, CryptoError> {
        let ctx =
            Pkcs11::new(config.module_path).map_err(|e| CryptoError::Internal(e.to_string()))?;
        ctx.initialize(CInitializeArgs::OsThreads)
            .map_err(|e| CryptoError::Internal(e.to_string()))?;

        let slots = ctx
            .get_slots_with_token()
            .map_err(|e| CryptoError::Internal(e.to_string()))?;
        let slot = slots
            .into_iter()
            .nth(config.slot_id as usize)
            .ok_or_else(|| CryptoError::Internal(format!("slot {} not found", config.slot_id)))?;

        let mut sessions = Vec::with_capacity(config.max_sessions);
        for _ in 0..config.max_sessions {
            let session = ctx
                .open_rw_session(slot)
                .map_err(|e| CryptoError::Internal(e.to_string()))?;
            session
                .login(cryptoki::session::UserType::User, Some(&config.pin))
                .map_err(|e| CryptoError::Internal(e.to_string()))?;
            sessions.push(session);
        }

        Ok(PkcsPool(Arc::new(Inner {
            ctx,
            slot,
            pin: config.pin,
            available: Mutex::new(sessions),
            max_sessions: config.max_sessions,
        })))
    }

    /// Borrow a session from the pool, opening a new one if all are in use.
    ///
    /// The returned [`PooledSession`] returns the session to the pool on drop.
    ///
    /// # Errors
    ///
    /// Returns [`CryptoError`] if a new session cannot be opened.
    pub fn get(&self) -> Result<PooledSession<'_>, CryptoError> {
        let session = {
            let mut guard = self.0.available.lock();
            guard.pop()
        };
        let session = if let Some(s) = session {
            s
        } else {
            // Pool exhausted; open a transient session.
            let s = self
                .0
                .ctx
                .open_rw_session(self.0.slot)
                .map_err(|e| CryptoError::Internal(e.to_string()))?;
            s.login(cryptoki::session::UserType::User, Some(&self.0.pin))
                .map_err(|e| CryptoError::Internal(e.to_string()))?;
            s
        };
        Ok(PooledSession {
            pool: self,
            session: Some(session),
        })
    }
}

/// A PKCS#11 session borrowed from [`PkcsPool`].
///
/// Returns the session to the pool on drop.
pub struct PooledSession<'a> {
    pool: &'a PkcsPool,
    session: Option<Session>,
}

impl<'a> PooledSession<'a> {
    /// Access the underlying PKCS#11 session.
    pub fn session(&self) -> &Session {
        self.session
            .as_ref()
            .expect("session always present while PooledSession is alive")
    }
}

impl<'a> Drop for PooledSession<'a> {
    fn drop(&mut self) {
        if let Some(s) = self.session.take() {
            let mut guard = self.pool.0.available.lock();
            if guard.len() < self.pool.0.max_sessions {
                guard.push(s);
            }
            // If over capacity, the session is dropped and closed.
        }
    }
}

/// Look up a key handle in the given session by its role label.
///
/// # Errors
///
/// Returns [`CryptoError::InvalidKey`] if the key is not found.
pub fn find_key_by_role(
    session: &Session,
    role: KeyRole,
) -> Result<cryptoki::object::ObjectHandle, CryptoError> {
    use cryptoki::object::{Attribute, AttributeType};

    let label = role.label();
    let template = vec![Attribute::Label(label.as_bytes().to_vec())];
    let handles = session
        .find_objects(&template)
        .map_err(|e| CryptoError::Internal(e.to_string()))?;
    handles
        .into_iter()
        .next()
        .ok_or_else(|| CryptoError::InvalidKey(format!("no key found with label {:?}", label)))
}
