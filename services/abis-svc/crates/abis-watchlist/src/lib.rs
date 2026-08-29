//! abis-watchlist crate.
#![warn(missing_docs)]
pub mod domain;
pub mod index;
pub mod manager;
pub mod matcher;

pub use domain::{Hit, MultiModalProbe, Watchlist, WatchlistClassification, WatchlistEntry};
pub use index::{BloomFilter, MultiIndex, WatchlistIndex};
pub use manager::{WatchlistError, WatchlistManager};
pub use matcher::{check_watchlist, check_watchlist_fast};
