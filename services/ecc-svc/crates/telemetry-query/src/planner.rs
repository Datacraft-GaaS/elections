//! Cost-model query planner.
//!
//! Estimates per-shard cardinality and selects the cheapest scan order for a
//! validated `TelemetryQuery`. The planner produces a `QueryPlan` that the
//! executor uses to drive partition pruning on `eel_telemetry_signals`.

use chrono::Datelike;
use uuid::Uuid;

use crate::dsl::TelemetryQuery;

// ---------------------------------------------------------------------------
// Plan types
// ---------------------------------------------------------------------------

/// Postgres partition month in YYYY-MM format.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Partition(pub String);

/// Chosen scan order from cheapest to most expensive.
#[derive(Debug, Clone)]
pub enum ScanOrder {
    /// Scan by device_id index first, then time partition.
    DeviceThenTime,
    /// Scan by time partition first, then device filter.
    TimeThenDevice,
    /// Full partition scan (no device filter, no geo filter).
    PartitionScan,
}

/// Planner output: which partitions to touch and in what order.
#[derive(Debug, Clone)]
pub struct QueryPlan {
    pub partitions: Vec<Partition>,
    pub scan_order: ScanOrder,
    pub estimated_rows: u64,
    pub device_ids: Vec<Uuid>,
}

// ---------------------------------------------------------------------------
// Planner
// ---------------------------------------------------------------------------

/// Lightweight cost-model planner. Uses heuristic cardinality estimates
/// derived from the time range width and number of device IDs.
pub struct CostModelPlanner;

impl CostModelPlanner {
    /// Produce a `QueryPlan` for a validated query.
    pub fn plan(q: &TelemetryQuery) -> QueryPlan {
        let partitions = Self::enumerate_partitions(q);
        let scan_order = Self::choose_scan_order(q, partitions.len());
        let estimated_rows = Self::estimate_rows(q, partitions.len());

        QueryPlan {
            partitions,
            scan_order,
            estimated_rows,
            device_ids: q.device_ids.clone(),
        }
    }

    /// Enumerate monthly partitions covered by the time range.
    fn enumerate_partitions(q: &TelemetryQuery) -> Vec<Partition> {
        let mut partitions = vec![];
        let mut current = q.time_range.from;
        let end = q.time_range.to;

        while current <= end {
            partitions.push(Partition(format!(
                "{:04}-{:02}",
                current.year(),
                current.month()
            )));
            // Advance to first day of next month
            let (y, m) = if current.month() == 12 {
                (current.year() + 1, 1)
            } else {
                (current.year(), current.month() + 1)
            };
            current = current
                .with_year(y)
                .and_then(|d| d.with_month(m))
                .and_then(|d| d.with_day(1))
                .unwrap_or(end + chrono::Duration::days(1));
        }

        partitions.sort();
        partitions.dedup();
        partitions
    }

    /// Choose cheapest scan order.
    ///
    /// If the query names <= 10 specific devices, device-first is cheapest
    /// because the device_id index is selective. Beyond that, partition-first
    /// wins because time partitioning prunes the most rows.
    fn choose_scan_order(q: &TelemetryQuery, partition_count: usize) -> ScanOrder {
        if q.device_ids.is_empty() {
            ScanOrder::PartitionScan
        } else if q.device_ids.len() <= 10 && partition_count <= 3 {
            ScanOrder::DeviceThenTime
        } else {
            ScanOrder::TimeThenDevice
        }
    }

    /// Heuristic row estimate.
    ///
    /// Baseline: ~1 M signals/device/day at full resolution; typical queries
    /// touch a 1-Hz downsample, so ~86_400 rows/device/day.
    fn estimate_rows(q: &TelemetryQuery, partition_count: usize) -> u64 {
        let days = (q.time_range.to - q.time_range.from).num_days().max(1) as u64;
        let devices = if q.device_ids.is_empty() {
            // Assume fleet of 65_000 devices; geo/class filters reduce this
            let geo_factor: u64 = if q.geo_polygon.is_some() { 100 } else { 65_000 };
            let class_factor: u64 = if q.signal_classes.is_empty() {
                13
            } else {
                q.signal_classes.len() as u64
            };
            geo_factor * class_factor
        } else {
            q.device_ids.len() as u64
        };

        // 86_400 rows/device/day across all classes; scale by class selection
        let class_ratio: f64 = if q.signal_classes.is_empty() {
            1.0
        } else {
            q.signal_classes.len() as f64 / 13.0
        };

        let raw = (devices as f64 * days as f64 * 86_400.0 * class_ratio) as u64;
        // Partitions not touched are free; cap extreme estimates
        let _ = partition_count;
        raw.min(5_000_000_000) // 5B row ceiling
    }
}
