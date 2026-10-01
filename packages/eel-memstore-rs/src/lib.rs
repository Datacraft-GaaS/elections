//! Thread-safe in-memory collections for services that have no store wired.
//!
//! Why this exists: a large number of eel handlers returned a success response
//! describing work they never did -- a created incident that existed nowhere, an
//! evidence bundle default-constructed for any id. The available fixes were to
//! delete the endpoints or refuse with 501, and both reduce capability. Neither
//! is right when the honest answer is "there is no store yet".
//!
//! This provides the missing capability: real create/read/update semantics, so
//! `POST /incidents` followed by `GET /incidents/{id}` returns what was posted,
//! and a lookup for something never created returns `None` rather than a
//! fabricated record. That is genuine behaviour a caller can rely on within the
//! process lifetime.
//!
//! What it is not: durable. State is lost on restart and is per-process, so it
//! cannot serve two replicas. That is a real limitation, stated here rather than
//! discovered later, and it is why [`Collection::is_durable`] exists as a name
//! services can surface in a health response. Where a service needs durability,
//! the sqlx `PgStore` pattern in `bvi-api/src/store.rs` is the target; this is
//! the step that lets a handler be correct before that store exists.
//!
//! Ordering is by key ([`BTreeMap`]) rather than insertion, so list endpoints
//! are stable across calls. A list whose order varies between two identical
//! requests is its own small lie.

use std::collections::BTreeMap;
use std::sync::RwLock;

use serde::{de::DeserializeOwned, Serialize};

/// A keyed, thread-safe collection of `T`.
#[derive(Debug)]
pub struct Collection<T> {
    inner: RwLock<BTreeMap<String, T>>,
}

impl<T> Default for Collection<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Collection<T> {
    /// An empty collection.
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(BTreeMap::new()),
        }
    }

    /// Number of stored items.
    pub fn len(&self) -> usize {
        self.inner.read().expect("memstore lock poisoned").len()
    }

    /// Whether the collection holds nothing.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Always false. This store is per-process and non-durable.
    ///
    /// Named so a service can report it rather than implying Postgres.
    pub fn is_durable(&self) -> bool {
        false
    }
}

impl<T: Clone> Collection<T> {
    /// Store `value` under `id`, returning whatever was there before.
    pub fn insert(&self, id: impl Into<String>, value: T) -> Option<T> {
        self.inner
            .write()
            .expect("memstore lock poisoned")
            .insert(id.into(), value)
    }

    /// Fetch by id. `None` means it was never created -- never a default record.
    pub fn get(&self, id: &str) -> Option<T> {
        self.inner
            .read()
            .expect("memstore lock poisoned")
            .get(id)
            .cloned()
    }

    /// Every stored value, ordered by key.
    pub fn list(&self) -> Vec<T> {
        self.inner
            .read()
            .expect("memstore lock poisoned")
            .values()
            .cloned()
            .collect()
    }

    /// Replace the value at `id` via `f`, if present. `None` if absent.
    pub fn update(&self, id: &str, f: impl FnOnce(T) -> T) -> Option<T> {
        let mut guard = self.inner.write().expect("memstore lock poisoned");
        let existing = guard.get_mut(id)?;
        let previous = existing.clone();
        *existing = f(previous.clone());
        Some(previous)
    }

    /// Apply `f` only if `pred` accepts the stored value.
    pub fn update_if(
        &self,
        id: &str,
        pred: impl FnOnce(&T) -> bool,
        f: impl FnOnce(T) -> T,
    ) -> Option<T> {
        let mut guard = self.inner.write().expect("memstore lock poisoned");
        let existing = guard.get(id)?;
        if !pred(existing) {
            return None;
        }
        let previous = existing.clone();
        guard.insert(id.to_string(), f(previous.clone()));
        Some(previous)
    }

    /// Remove by id, returning the value if it was there.
    pub fn remove(&self, id: &str) -> Option<T> {
        self.inner
            .write()
            .expect("memstore lock poisoned")
            .remove(id)
    }

    /// Values matching `pred`, in key order.
    pub fn list_where(&self, pred: impl Fn(&T) -> bool) -> Vec<T> {
        self.inner
            .read()
            .expect("memstore lock poisoned")
            .values()
            .filter(|v| pred(v))
            .cloned()
            .collect()
    }
}

/// Serialisation round-trip, so a service can persist a collection's contents
/// without depending on this crate's internals. The point is recoverability:
/// in-memory state that cannot be written out is state that is lost on deploy.
pub trait Dumpable: Sized {
    /// All items with their keys.
    fn dump(&self) -> BTreeMap<String, Self>;
    /// Replace all contents.
    fn restore(&self, items: BTreeMap<String, Self>);
}

impl<T: Clone + Serialize + DeserializeOwned> Collection<T> {
    /// Snapshot every item as JSON.
    pub fn to_map(&self) -> BTreeMap<String, serde_json::Value>
    where
        T: Serialize,
    {
        self.inner
            .read()
            .expect("memstore lock poisoned")
            .iter()
            .map(|(k, v)| {
                (
                    k.clone(),
                    serde_json::to_value(v).expect("memstore value must serialise"),
                )
            })
            .collect()
    }

    /// Replace all contents from a JSON snapshot. Entries that fail to
    /// deserialise are skipped rather than aborting the restore.
    pub fn from_map(&self, items: &BTreeMap<String, serde_json::Value>)
    where
        T: DeserializeOwned,
    {
        let mut guard = self.inner.write().expect("memstore lock poisoned");
        guard.clear();
        for (k, v) in items {
            if let Ok(parsed) = serde_json::from_value::<T>(v.clone()) {
                guard.insert(k.clone(), parsed);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
    struct Incident {
        id: String,
        severity: String,
        resolved: bool,
    }

    fn store() -> Collection<Incident> {
        Collection::new()
    }

    fn inc(id: &str) -> Incident {
        Incident {
            id: id.to_string(),
            severity: "high".into(),
            resolved: false,
        }
    }

    #[test]
    fn insert_then_get_returns_what_was_stored() {
        let s = store();
        s.insert("i-1", inc("i-1"));
        assert_eq!(s.get("i-1"), Some(inc("i-1")));
        assert_eq!(s.len(), 1);
    }

    #[test]
    fn get_unknown_id_is_none_not_a_default_record() {
        // The behaviour that replaces fabrication: a lookup for something never
        // created returns None, so the handler can 404 instead of echoing a
        // default-constructed record back as if it existed.
        let s = store();
        assert_eq!(s.get("never-created"), None);
        assert!(s.is_empty());
    }

    #[test]
    fn list_is_ordered_by_key_and_stable_across_calls() {
        let s = store();
        s.insert("i-3", inc("i-3"));
        s.insert("i-1", inc("i-1"));
        s.insert("i-2", inc("i-2"));
        let first: Vec<String> = s.list().into_iter().map(|i| i.id).collect();
        let second: Vec<String> = s.list().into_iter().map(|i| i.id).collect();
        assert_eq!(first, vec!["i-1", "i-2", "i-3"]);
        assert_eq!(first, second, "list order must not vary between calls");
    }

    #[test]
    fn insert_replaces_and_returns_previous() {
        let s = store();
        s.insert("i-1", inc("i-1"));
        let mut changed = inc("i-1");
        changed.severity = "critical".into();
        assert_eq!(s.insert("i-1", changed.clone()), Some(inc("i-1")));
        assert_eq!(s.get("i-1"), Some(changed));
        assert_eq!(s.len(), 1, "replacing must not grow the collection");
    }

    #[test]
    fn update_only_applies_to_present_items() {
        let s = store();
        s.insert("i-1", inc("i-1"));
        let prev = s.update("i-1", |mut i| {
            i.resolved = true;
            i
        });
        assert!(prev.is_some());
        assert_eq!(s.get("i-1").unwrap().resolved, true);
        assert!(
            s.update("absent", |i| i).is_none(),
            "updating an absent id must report absence, not create it"
        );
        assert_eq!(s.len(), 1);
    }

    #[test]
    fn update_if_respects_the_predicate() {
        let s = store();
        s.insert("i-1", inc("i-1"));
        assert!(
            s.update_if("i-1", |i| i.resolved, |mut i| {
                i.resolved = true;
                i
            })
            .is_none(),
            "a guarded update must refuse when the predicate fails"
        );
        assert_eq!(s.get("i-1").unwrap().resolved, false);
    }

    #[test]
    fn remove_reports_whether_anything_was_there() {
        let s = store();
        s.insert("i-1", inc("i-1"));
        assert_eq!(s.remove("i-1"), Some(inc("i-1")));
        assert_eq!(s.remove("i-1"), None);
        assert!(s.is_empty());
    }

    #[test]
    fn list_where_filters_without_removing() {
        let s = store();
        s.insert("i-1", inc("i-1"));
        let mut other = inc("i-2");
        other.severity = "low".into();
        s.insert("i-2", other);
        assert_eq!(s.list_where(|i| i.severity == "high").len(), 1);
        assert_eq!(s.len(), 2);
    }

    #[test]
    fn is_durable_is_false_and_says_so() {
        assert!(!store().is_durable());
    }

    #[test]
    fn round_trips_through_a_json_snapshot() {
        let s = store();
        s.insert("i-1", inc("i-1"));
        s.insert("i-2", inc("i-2"));
        let snapshot = s.to_map();
        assert_eq!(snapshot.len(), 2);

        let restored: Collection<Incident> = Collection::new();
        restored.from_map(&snapshot);
        assert_eq!(restored.len(), 2);
        assert_eq!(restored.get("i-1"), Some(inc("i-1")));
    }

    #[test]
    fn restore_skips_unparseable_entries_rather_than_failing() {
        let mut items = BTreeMap::new();
        items.insert(
            "i-1".to_string(),
            serde_json::json!({"id": "i-1", "severity": "high", "resolved": false}),
        );
        items.insert("i-bad".to_string(), serde_json::json!({"nonsense": true}));
        let s: Collection<Incident> = Collection::new();
        s.from_map(&items);
        assert_eq!(s.len(), 1);
        assert_eq!(s.get("i-1"), Some(inc("i-1")));
    }

    #[test]
    fn is_send_and_sync_so_it_can_live_in_axum_state() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<Collection<Incident>>();
    }
}