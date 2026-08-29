//! gazette crate.
#![warn(missing_docs)]
pub mod api;
pub mod domain;
pub mod events;
pub mod rules;

pub use domain::{
    CalendarEvent, CalendarEventType, GazetteNotice, NoticeType, StatutoryRule, Subscription,
    SubscriptionChannel,
};
pub use rules::derive_notices;

pub mod spec;
pub mod spec_handlers;
