use chrono::Utc;

use crate::{
    domain::{Hit, MultiModalProbe, Watchlist, WatchlistEntry},
    index::MultiIndex,
};

pub fn check_watchlist(
    probe: &MultiModalProbe,
    watchlists: &[Watchlist],
    index: &MultiIndex,
    entries: &[WatchlistEntry],
) -> Vec<Hit> {
    let mut hits = Vec::new();

    for (modality, hash) in &probe.template_hashes {
        // Walk each watchlist's index for this modality.
        for wl in watchlists {
            if let Some(idx) = index.get_index(&wl.watchlist_id, modality) {
                if idx.contains(hash) {
                    // Exact match confirmed; locate the entry.
                    if let Some(entry) = entries
                        .iter()
                        .find(|e| &e.template_hash == hash && e.watchlist_id == wl.watchlist_id)
                    {
                        if !entry.is_sunsetted() {
                            hits.push(Hit {
                                entry_id: entry.entry_id.clone(),
                                watchlist_id: wl.watchlist_id.clone(),
                                watchlist_name: wl.name.clone(),
                                subject_ref: entry.subject_ref.clone(),
                                modality: modality.clone(),
                                similarity: 1.0,
                                hit_at: Utc::now(),
                            });
                        }
                    }
                }
            }
        }
    }

    hits
}

pub fn check_watchlist_fast(template_hash: &str, modality: &str, index: &MultiIndex) -> bool {
    index
        .indices_for_modality(modality)
        .any(|idx| idx.contains(template_hash))
}
