# eel-abis-watchlist

Multi-modal fast-precheck watchlist crate for the EEL ABIS stack.

## Overview

Provides a sub-100 microsecond contains-check against watchlist entries using a two-tier index: a Bloom filter for probabilistic fast rejection, backed by a HashSet for exact confirmation. Mirrors the `eel_watchlists` and `eel_watchlist_entries` tables from V030.

## Components

### Index (src/index.rs)

`BloomFilter`: double-hashing (FNV-1a + djb2) with configurable capacity and target false-positive rate. Bits stored as Vec<u64>.

`WatchlistIndex`: combines Bloom + HashSet per (watchlist_id, modality) pair. Insert/remove/contains operations. Remove keeps Bloom intact (no false negatives introduced - the HashSet is authoritative for removes).

`MultiIndex`: collection of WatchlistIndex instances. `contains_any` scans by modality.

### Matcher (src/matcher.rs)

`check_watchlist(probe, watchlists, index, entries)`: for each (modality, hash) in the multi-modal probe, runs bloom then exact lookup. Returns `Vec<Hit>` with similarity=1.0 for exact hash matches.

`check_watchlist_fast(hash, modality, index)`: bloom-only check, returns bool.

### Manager (src/manager.rs)

`WatchlistManager` wraps state in `Arc<RwLock<_>>` for concurrent access:

- `add_entry`: validates capacity, inserts into HashSet index and Bloom filter.
- `remove_entry`: removes from HashSet (Bloom remains, false positives tolerated).
- `sunset_entry`: sets sunset_at timestamp.
- `purge_sunsetted`: removes all entries past their sunset time.
- `record_hit`: increments hit_count and sets last_hit_at.

## Watchlist Classifications

Mirrors V030 CHECK constraint: `exclusion`, `duplicate_alert`, `vip_flag`, `fraud_hold`.

## Example Usage

```rust
use eel_abis_watchlist::{manager::WatchlistManager, domain::{MultiModalProbe, Watchlist}};

let mgr = WatchlistManager::new();
mgr.add_entry("wl-id", entry)?;

let probe = MultiModalProbe {
    probe_id: uuid,
    template_hashes: vec![("fingerprint".into(), hash)],
};
let hits = mgr.check(&probe, &watchlists);
```
