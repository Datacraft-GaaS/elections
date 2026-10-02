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

use chrono::{DateTime, Utc};
use std::sync::{Arc, RwLock};

use crate::{
    domain::{Hit, MultiModalProbe, Watchlist, WatchlistEntry},
    index::{MultiIndex, WatchlistIndex},
    matcher::check_watchlist,
};

#[derive(Debug, thiserror::Error)]
pub enum WatchlistError {
    #[error("watchlist not found: {0}")]
    WatchlistNotFound(String),
    #[error("entry not found: {0}")]
    EntryNotFound(String),
    #[error("watchlist at capacity")]
    AtCapacity,
    #[error("lock poisoned")]
    LockPoisoned,
}

pub struct WatchlistManager {
    watchlists: Arc<RwLock<Vec<Watchlist>>>,
    entries: Arc<RwLock<Vec<WatchlistEntry>>>,
    index: Arc<RwLock<MultiIndex>>,
}

impl WatchlistManager {
    pub fn new() -> Self {
        Self {
            watchlists: Arc::new(RwLock::new(Vec::new())),
            entries: Arc::new(RwLock::new(Vec::new())),
            index: Arc::new(RwLock::new(MultiIndex::new())),
        }
    }

    pub fn add_watchlist(&self, wl: Watchlist) -> Result<(), WatchlistError> {
        let mut wls = self
            .watchlists
            .write()
            .map_err(|_| WatchlistError::LockPoisoned)?;
        let mut idx = self
            .index
            .write()
            .map_err(|_| WatchlistError::LockPoisoned)?;
        // Create indices for each modality declared on the watchlist.
        for modality in &wl.modalities {
            if idx.get_index(&wl.watchlist_id, modality).is_none() {
                idx.add_index(WatchlistIndex::new(
                    wl.watchlist_id.clone(),
                    modality.clone(),
                    wl.max_size as usize,
                ));
            }
        }
        wls.push(wl);
        Ok(())
    }

    pub fn add_entry(
        &self,
        watchlist_id: &str,
        entry: WatchlistEntry,
    ) -> Result<(), WatchlistError> {
        // Capacity check.
        {
            let wls = self
                .watchlists
                .read()
                .map_err(|_| WatchlistError::LockPoisoned)?;
            let wl = wls
                .iter()
                .find(|w| w.watchlist_id == watchlist_id)
                .ok_or_else(|| WatchlistError::WatchlistNotFound(watchlist_id.to_string()))?;
            let entries = self
                .entries
                .read()
                .map_err(|_| WatchlistError::LockPoisoned)?;
            let current = entries
                .iter()
                .filter(|e| e.watchlist_id == watchlist_id)
                .count();
            if current >= wl.max_size as usize {
                return Err(WatchlistError::AtCapacity);
            }
        }

        let hash = entry.template_hash.clone();
        let modality = entry.modality.clone();

        let mut entries = self
            .entries
            .write()
            .map_err(|_| WatchlistError::LockPoisoned)?;
        let mut idx = self
            .index
            .write()
            .map_err(|_| WatchlistError::LockPoisoned)?;

        // Ensure an index exists for this watchlist/modality pair.
        if idx.get_index(watchlist_id, &modality).is_none() {
            idx.add_index(WatchlistIndex::new(
                watchlist_id.to_string(),
                modality.clone(),
                1024,
            ));
        }

        if let Some(wi) = idx.get_index_mut(watchlist_id, &modality) {
            wi.insert(hash);
        }

        entries.push(entry);
        Ok(())
    }

    pub fn remove_entry(&self, entry_id: &str) -> Result<(), WatchlistError> {
        let mut entries = self
            .entries
            .write()
            .map_err(|_| WatchlistError::LockPoisoned)?;
        let pos = entries
            .iter()
            .position(|e| e.entry_id == entry_id)
            .ok_or_else(|| WatchlistError::EntryNotFound(entry_id.to_string()))?;

        let entry = entries.remove(pos);

        let mut idx = self
            .index
            .write()
            .map_err(|_| WatchlistError::LockPoisoned)?;
        if let Some(wi) = idx.get_index_mut(&entry.watchlist_id, &entry.modality) {
            wi.remove(&entry.template_hash);
        }

        Ok(())
    }

    pub fn sunset_entry(
        &self,
        entry_id: &str,
        sunset_at: DateTime<Utc>,
    ) -> Result<(), WatchlistError> {
        let mut entries = self
            .entries
            .write()
            .map_err(|_| WatchlistError::LockPoisoned)?;
        let entry = entries
            .iter_mut()
            .find(|e| e.entry_id == entry_id)
            .ok_or_else(|| WatchlistError::EntryNotFound(entry_id.to_string()))?;
        entry.sunset_at = Some(sunset_at);
        Ok(())
    }

    pub fn record_hit(&self, entry_id: &str) -> Result<(), WatchlistError> {
        let mut entries = self
            .entries
            .write()
            .map_err(|_| WatchlistError::LockPoisoned)?;
        let entry = entries
            .iter_mut()
            .find(|e| e.entry_id == entry_id)
            .ok_or_else(|| WatchlistError::EntryNotFound(entry_id.to_string()))?;
        entry.hit_count += 1;
        entry.last_hit_at = Some(Utc::now());
        Ok(())
    }

    pub fn entries_count(&self) -> usize {
        self.entries.read().map(|e| e.len()).unwrap_or(0)
    }

    pub fn purge_sunsetted(&self) -> usize {
        let mut entries = match self.entries.write() {
            Ok(e) => e,
            Err(_) => return 0,
        };
        let before = entries.len();
        entries.retain(|e| !e.is_sunsetted());
        before - entries.len()
    }

    pub fn check(&self, probe: &MultiModalProbe, watchlists: &[Watchlist]) -> Vec<Hit> {
        let entries = match self.entries.read() {
            Ok(e) => e,
            Err(_) => return Vec::new(),
        };
        let idx = match self.index.read() {
            Ok(i) => i,
            Err(_) => return Vec::new(),
        };
        check_watchlist(probe, watchlists, &idx, &entries)
    }
}

impl Default for WatchlistManager {
    fn default() -> Self {
        Self::new()
    }
}
