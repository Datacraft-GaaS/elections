use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Shed metrics per priority class.
#[derive(Debug, Default, Clone)]
pub struct ShedMetrics {
    pub shed_anonymous: Arc<AtomicU64>,
    pub shed_accredited_machine: Arc<AtomicU64>,
    pub shed_authenticated_user: Arc<AtomicU64>,
    pub shed_admin: Arc<AtomicU64>,
    pub target_breach_count: Arc<AtomicU64>,
    pub current_queue_depth: Arc<AtomicU64>,
}

impl ShedMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_shed(&self, priority: crate::priority::Priority) {
        use crate::priority::Priority;
        let counter = match priority {
            Priority::Anonymous => &self.shed_anonymous,
            Priority::AccreditedMachine => &self.shed_accredited_machine,
            Priority::AuthenticatedUser => &self.shed_authenticated_user,
            Priority::Admin => &self.shed_admin,
        };
        counter.fetch_add(1, Ordering::Relaxed);
        tracing::info!(priority = priority.name(), "request_shed");
    }

    pub fn record_breach(&self) {
        self.target_breach_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn set_queue_depth(&self, depth: u64) {
        self.current_queue_depth.store(depth, Ordering::Relaxed);
    }

    pub fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            shed_anonymous: self.shed_anonymous.load(Ordering::Relaxed),
            shed_accredited_machine: self.shed_accredited_machine.load(Ordering::Relaxed),
            shed_authenticated_user: self.shed_authenticated_user.load(Ordering::Relaxed),
            shed_admin: self.shed_admin.load(Ordering::Relaxed),
            target_breach_count: self.target_breach_count.load(Ordering::Relaxed),
            current_queue_depth: self.current_queue_depth.load(Ordering::Relaxed),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct MetricsSnapshot {
    pub shed_anonymous: u64,
    pub shed_accredited_machine: u64,
    pub shed_authenticated_user: u64,
    pub shed_admin: u64,
    pub target_breach_count: u64,
    pub current_queue_depth: u64,
}
