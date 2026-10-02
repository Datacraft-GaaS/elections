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

use chrono::{Duration, Utc};
use eel_abis_watchlist::{
    domain::{MultiModalProbe, Watchlist, WatchlistClassification, WatchlistEntry},
    index::{BloomFilter, MultiIndex, WatchlistIndex},
    manager::WatchlistManager,
    matcher::{check_watchlist, check_watchlist_fast},
};
use std::sync::Arc;

fn make_watchlist(id: &str) -> Watchlist {
    Watchlist {
        watchlist_id: id.to_string(),
        name: format!("list-{id}"),
        description: None,
        modalities: vec!["ear".to_string()],
        max_size: 10_000,
        current_size: 0,
        classification: WatchlistClassification::Exclusion,
        owner: "test".to_string(),
        created_at: Utc::now(),
        last_reviewed_at: None,
        suspended_at: None,
    }
}

fn make_entry(entry_id: &str, watchlist_id: &str, hash: &str) -> WatchlistEntry {
    WatchlistEntry {
        entry_id: entry_id.to_string(),
        watchlist_id: watchlist_id.to_string(),
        subject_ref: "sub-1".to_string(),
        modality: "ear".to_string(),
        template_hash: hash.to_string(),
        hit_count: 0,
        last_hit_at: None,
        added_at: Utc::now(),
        sunset_at: None,
        notes: None,
    }
}

// --- BloomFilter ---

#[test]
fn bloom_insert_then_may_contain() {
    let mut bf = BloomFilter::new(100, 0.01);
    bf.insert("hello");
    assert!(bf.may_contain("hello"));
}

#[test]
fn bloom_fresh_does_not_contain() {
    let bf = BloomFilter::new(100, 0.01);
    assert!(!bf.may_contain("not-inserted"));
}

// --- WatchlistIndex ---

#[test]
fn wl_index_contains_after_insert() {
    let mut idx = WatchlistIndex::new("wl1".to_string(), "ear".to_string(), 100);
    idx.insert("abc123".to_string());
    assert!(idx.contains("abc123"));
}

#[test]
fn wl_index_not_contains_after_remove() {
    let mut idx = WatchlistIndex::new("wl1".to_string(), "ear".to_string(), 100);
    idx.insert("abc123".to_string());
    idx.remove("abc123");
    assert!(!idx.contains("abc123"));
}

#[test]
fn wl_index_len_tracks_correctly() {
    let mut idx = WatchlistIndex::new("wl1".to_string(), "ear".to_string(), 100);
    assert_eq!(idx.len(), 0);
    idx.insert("h1".to_string());
    idx.insert("h2".to_string());
    assert_eq!(idx.len(), 2);
    idx.remove("h1");
    assert_eq!(idx.len(), 1);
}

// --- check_watchlist ---

#[test]
fn check_watchlist_finds_known_hash() {
    let wl = make_watchlist("wl1");
    let entry = make_entry("e1", "wl1", "deadbeef");

    let mut multi = MultiIndex::new();
    let mut widx = WatchlistIndex::new("wl1".to_string(), "ear".to_string(), 100);
    widx.insert("deadbeef".to_string());
    multi.add_index(widx);

    let probe = MultiModalProbe {
        probe_id: "p1".to_string(),
        template_hashes: vec![("ear".to_string(), "deadbeef".to_string())],
    };

    let hits = check_watchlist(&probe, &[wl], &multi, &[entry]);
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].entry_id, "e1");
}

#[test]
fn check_watchlist_returns_empty_for_unknown() {
    let wl = make_watchlist("wl1");
    let multi = MultiIndex::new();
    let probe = MultiModalProbe {
        probe_id: "p1".to_string(),
        template_hashes: vec![("ear".to_string(), "unknown".to_string())],
    };
    let hits = check_watchlist(&probe, &[wl], &multi, &[]);
    assert!(hits.is_empty());
}

#[test]
fn check_watchlist_fast_bloom_check_works() {
    let mut multi = MultiIndex::new();
    let mut widx = WatchlistIndex::new("wl1".to_string(), "ear".to_string(), 100);
    widx.insert("cafebabe".to_string());
    multi.add_index(widx);

    assert!(check_watchlist_fast("cafebabe", "ear", &multi));
    assert!(!check_watchlist_fast("nothere", "ear", &multi));
}

// --- WatchlistManager ---

fn make_manager_with_wl() -> (WatchlistManager, Watchlist) {
    let mgr = WatchlistManager::new();
    let wl = make_watchlist("wl1");
    mgr.add_watchlist(wl.clone()).expect("add watchlist");
    (mgr, wl)
}

#[test]
fn manager_add_entry_increases_count() {
    let (mgr, _wl) = make_manager_with_wl();
    mgr.add_entry("wl1", make_entry("e1", "wl1", "hash1"))
        .expect("add");
    assert_eq!(mgr.entries_count(), 1);
}

#[test]
fn manager_remove_entry_decreases_count() {
    let (mgr, _wl) = make_manager_with_wl();
    mgr.add_entry("wl1", make_entry("e1", "wl1", "hash1"))
        .expect("add");
    mgr.remove_entry("e1").expect("remove");
    assert_eq!(mgr.entries_count(), 0);
}

#[test]
fn manager_sunset_entry_sets_sunset_at() {
    let (mgr, _wl) = make_manager_with_wl();
    mgr.add_entry("wl1", make_entry("e1", "wl1", "hash1"))
        .expect("add");
    let t = Utc::now() - Duration::seconds(1);
    mgr.sunset_entry("e1", t).expect("sunset");
    // Verify it appears sunsetted.
    assert_eq!(mgr.purge_sunsetted(), 1);
}

#[test]
fn manager_purge_sunsetted_removes_entries() {
    let (mgr, _wl) = make_manager_with_wl();
    mgr.add_entry("wl1", make_entry("e1", "wl1", "h1"))
        .expect("add");
    mgr.add_entry("wl1", make_entry("e2", "wl1", "h2"))
        .expect("add");
    let past = Utc::now() - Duration::seconds(10);
    mgr.sunset_entry("e1", past).expect("sunset");
    let purged = mgr.purge_sunsetted();
    assert_eq!(purged, 1);
    assert_eq!(mgr.entries_count(), 1);
}

#[test]
fn manager_record_hit_increments() {
    let (mgr, _wl) = make_manager_with_wl();
    mgr.add_entry("wl1", make_entry("e1", "wl1", "h1"))
        .expect("add");
    mgr.record_hit("e1").expect("hit");
    mgr.record_hit("e1").expect("hit");
    // We can't read internal hit_count directly; just ensure no error.
}

#[test]
fn multi_index_remove_watchlist_removes_all() {
    let mut multi = MultiIndex::new();
    multi.add_index(WatchlistIndex::new(
        "wl1".to_string(),
        "ear".to_string(),
        100,
    ));
    multi.add_index(WatchlistIndex::new(
        "wl1".to_string(),
        "gait".to_string(),
        100,
    ));
    multi.add_index(WatchlistIndex::new(
        "wl2".to_string(),
        "ear".to_string(),
        100,
    ));
    multi.remove_watchlist("wl1");
    assert!(multi.get_index("wl1", "ear").is_none());
    assert!(multi.get_index("wl1", "gait").is_none());
    assert!(multi.get_index("wl2", "ear").is_some());
}

#[test]
fn contains_check_10k_entries() {
    let mut idx = WatchlistIndex::new("wl1".to_string(), "ear".to_string(), 10_000);
    let hashes: Vec<String> = (0..10_000).map(|i| format!("hash-{i:08x}")).collect();
    for h in &hashes {
        idx.insert(h.clone());
    }
    let missing = hashes.iter().filter(|h| !idx.contains(h)).count();
    assert_eq!(missing, 0, "all 10k inserted hashes should be found");
}

#[test]
fn watchlist_entry_is_sunsetted_past_date() {
    let mut entry = make_entry("e1", "wl1", "h1");
    entry.sunset_at = Some(Utc::now() - Duration::seconds(1));
    assert!(entry.is_sunsetted());
}

#[test]
fn concurrent_record_hit_thread_safety() {
    let mgr = Arc::new(WatchlistManager::new());
    let wl = make_watchlist("wl1");
    mgr.add_watchlist(wl).expect("add wl");
    mgr.add_entry("wl1", make_entry("e1", "wl1", "h1"))
        .expect("add entry");

    let handles: Vec<_> = (0..8)
        .map(|_| {
            let m = Arc::clone(&mgr);
            std::thread::spawn(move || {
                for _ in 0..100 {
                    m.record_hit("e1").expect("hit");
                }
            })
        })
        .collect();

    for h in handles {
        h.join().expect("thread join");
    }

    assert_eq!(mgr.entries_count(), 1);
}
