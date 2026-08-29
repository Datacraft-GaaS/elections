//! Store for inventory-svc: an enum over Postgres (sqlx) and an in-memory
//! fake. Handlers stay backend-agnostic; the in-memory arm keeps the service
//! and its tests hermetic.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use chrono::{DateTime, Utc};
use serde::de::DeserializeOwned;
use serde::Serialize;
use sqlx::PgPool;

use custody::domain::{Asset, CustodyEvent};
use transport::domain::{SecurityAssignment, Vehicle, VehicleLoad};
use warehouse::domain::{Incident, Pallet, Warehouse, WarehouseTransfer};

// ---------------------------------------------------------------------------
// Enum <-> string helpers (serde round-trip; the enums derive snake_case)
// ---------------------------------------------------------------------------

fn en<T: Serialize>(v: &T) -> String {
    match serde_json::to_value(v) {
        Ok(serde_json::Value::String(s)) => s,
        _ => String::new(),
    }
}

fn de<T: DeserializeOwned + Default>(s: &str) -> T {
    serde_json::from_value(serde_json::Value::String(s.to_string())).unwrap_or_default()
}

// ---------------------------------------------------------------------------
// In-memory data
// ---------------------------------------------------------------------------

/// In-memory aggregate state (private fields; an opaque handle when the
/// `Mem` store arm is used by tests).
#[derive(Default)]
pub struct Data {
    vehicles: HashMap<String, Vehicle>,
    loads: HashMap<String, VehicleLoad>,
    assignments: Vec<SecurityAssignment>,
    warehouses: HashMap<String, Warehouse>,
    transfers: HashMap<String, WarehouseTransfer>,
    pallets: HashMap<String, Pallet>,
    incidents: HashMap<String, Incident>,
    assets: HashMap<String, Asset>,
    custody_events: Vec<CustodyEvent>,
    shortfalls: HashMap<String, crate::spec::Shortfall>,
}

// ---------------------------------------------------------------------------
// Store enum
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub enum Store {
    Pg(PgPool),
    Mem(Arc<Mutex<Data>>),
}

impl Default for Store {
    fn default() -> Self {
        Store::Mem(Arc::new(Mutex::new(Data::default())))
    }
}

impl Store {
    // -- vehicles ---------------------------------------------------------

    pub async fn insert_vehicle(&self, v: &Vehicle) -> Result<(), sqlx::Error> {
        match self {
            Store::Pg(pool) => {
                sqlx::query(
                    "INSERT INTO vehicles (id, registration, vehicle_type, capacity, status) \
                     VALUES ($1, $2, $3, $4, $5) \
                     ON CONFLICT (id) DO UPDATE SET registration = $2, vehicle_type = $3, \
                       capacity = $4, status = $5",
                )
                .bind(&v.id)
                .bind(&v.registration)
                .bind(en(&v.vehicle_type))
                .bind(v.capacity as i64)
                .bind(en(&v.status))
                .execute(pool)
                .await?;
                Ok(())
            }
            Store::Mem(m) => {
                m.lock()
                    .unwrap_or_else(|p| p.into_inner())
                    .vehicles
                    .insert(v.id.clone(), v.clone());
                Ok(())
            }
        }
    }

    pub async fn get_vehicle(&self, id: &str) -> Result<Option<Vehicle>, sqlx::Error> {
        match self {
            Store::Pg(pool) => {
                let row = sqlx::query_as::<_, (String, String, String, i64, String)>(
                    "SELECT id, registration, vehicle_type, capacity, status FROM vehicles WHERE id = $1",
                )
                .bind(id)
                .fetch_optional(pool)
                .await?;
                Ok(row.map(
                    |(id, registration, vehicle_type, capacity, status)| Vehicle {
                        id,
                        registration,
                        vehicle_type: de(&vehicle_type),
                        capacity: capacity as u64,
                        status: de(&status),
                    },
                ))
            }
            Store::Mem(m) => Ok(m
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .vehicles
                .get(id)
                .cloned()),
        }
    }

    pub async fn list_vehicles(&self) -> Result<Vec<Vehicle>, sqlx::Error> {
        match self {
            Store::Pg(pool) => {
                let rows = sqlx::query_as::<_, (String, String, String, i64, String)>(
                    "SELECT id, registration, vehicle_type, capacity, status FROM vehicles ORDER BY id",
                )
                .fetch_all(pool)
                .await?;
                Ok(rows
                    .into_iter()
                    .map(
                        |(id, registration, vehicle_type, capacity, status)| Vehicle {
                            id,
                            registration,
                            vehicle_type: de(&vehicle_type),
                            capacity: capacity as u64,
                            status: de(&status),
                        },
                    )
                    .collect())
            }
            Store::Mem(m) => Ok(m
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .vehicles
                .values()
                .cloned()
                .collect()),
        }
    }

    // -- loads ------------------------------------------------------------

    pub async fn insert_load(&self, l: &VehicleLoad) -> Result<(), sqlx::Error> {
        match self {
            Store::Pg(pool) => {
                let mut tx = pool.begin().await?;
                sqlx::query(
                    "INSERT INTO vehicle_loads (id, vehicle_id, destination, escort_agent_id, status, \
                       loaded_at, dispatched_at, delivered_at) \
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8) \
                     ON CONFLICT (id) DO UPDATE SET vehicle_id = $2, destination = $3, \
                       escort_agent_id = $4, status = $5, loaded_at = $6, dispatched_at = $7, \
                       delivered_at = $8",
                )
                .bind(&l.id)
                .bind(&l.vehicle_id)
                .bind(&l.destination)
                .bind(&l.escort_agent_id)
                .bind(en(&l.status))
                .bind(l.loaded_at)
                .bind(l.dispatched_at)
                .bind(l.delivered_at)
                .execute(&mut *tx)
                .await?;
                sqlx::query("DELETE FROM vehicle_load_items WHERE load_id = $1")
                    .bind(&l.id)
                    .execute(&mut *tx)
                    .await?;
                for item in &l.items {
                    sqlx::query(
                        "INSERT INTO vehicle_load_items (id, load_id, lot_id, quantity, scan_method, \
                           scan_value, scanned_by, scanned_at) \
                         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
                    )
                    .bind(&item.id)
                    .bind(&l.id)
                    .bind(&item.lot_id)
                    .bind(item.quantity as i64)
                    .bind(en(&item.scan_method))
                    .bind(&item.scan_value)
                    .bind(&item.scanned_by)
                    .bind(item.scanned_at)
                    .execute(&mut *tx)
                    .await?;
                }
                tx.commit().await?;
                Ok(())
            }
            Store::Mem(m) => {
                m.lock()
                    .unwrap_or_else(|p| p.into_inner())
                    .loads
                    .insert(l.id.clone(), l.clone());
                Ok(())
            }
        }
    }

    pub async fn get_load(&self, id: &str) -> Result<Option<VehicleLoad>, sqlx::Error> {
        match self {
            Store::Pg(pool) => {
                let row = sqlx::query_as::<_, (String, String, String, Option<String>, String, DateTime<Utc>, Option<DateTime<Utc>>, Option<DateTime<Utc>>)>(
                    "SELECT id, vehicle_id, destination, escort_agent_id, status, loaded_at, dispatched_at, delivered_at \
                     FROM vehicle_loads WHERE id = $1",
                )
                .bind(id)
                .fetch_optional(pool)
                .await?;
                let Some((
                    id,
                    vehicle_id,
                    destination,
                    escort_agent_id,
                    status,
                    loaded_at,
                    dispatched_at,
                    delivered_at,
                )) = row
                else {
                    return Ok(None);
                };
                let items = sqlx::query_as::<
                    _,
                    (String, String, i64, String, String, String, DateTime<Utc>),
                >(
                    "SELECT id, lot_id, quantity, scan_method, scan_value, scanned_by, scanned_at \
                     FROM vehicle_load_items WHERE load_id = $1 ORDER BY id",
                )
                .bind(&id)
                .fetch_all(pool)
                .await?
                .into_iter()
                .map(
                    |(id, lot_id, quantity, scan_method, scan_value, scanned_by, scanned_at)| {
                        transport::domain::LoadItem {
                            id,
                            lot_id,
                            quantity: quantity as u64,
                            scan_method: de(&scan_method),
                            scan_value,
                            scanned_by,
                            scanned_at,
                        }
                    },
                )
                .collect();
                Ok(Some(VehicleLoad {
                    id,
                    vehicle_id,
                    destination,
                    escort_agent_id,
                    status: de(&status),
                    items,
                    loaded_at,
                    dispatched_at,
                    delivered_at,
                }))
            }
            Store::Mem(m) => Ok(m
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .loads
                .get(id)
                .cloned()),
        }
    }

    // -- security assignments ---------------------------------------------

    pub async fn insert_assignment(&self, a: &SecurityAssignment) -> Result<(), sqlx::Error> {
        match self {
            Store::Pg(pool) => {
                sqlx::query(
                    "INSERT INTO security_assignments (id, agent_id, target, target_id, starts_at, ends_at, status) \
                     VALUES ($1, $2, $3, $4, $5, $6, $7)",
                )
                .bind(&a.id)
                .bind(&a.agent_id)
                .bind(en(&a.target))
                .bind(&a.target_id)
                .bind(a.starts_at)
                .bind(a.ends_at)
                .bind(en(&a.status))
                .execute(pool)
                .await?;
                Ok(())
            }
            Store::Mem(m) => {
                m.lock()
                    .unwrap_or_else(|p| p.into_inner())
                    .assignments
                    .push(a.clone());
                Ok(())
            }
        }
    }

    pub async fn list_assignments(&self) -> Result<Vec<SecurityAssignment>, sqlx::Error> {
        match self {
            Store::Pg(pool) => {
                let rows = sqlx::query_as::<
                    _,
                    (
                        String,
                        String,
                        String,
                        String,
                        DateTime<Utc>,
                        DateTime<Utc>,
                        String,
                    ),
                >(
                    "SELECT id, agent_id, target, target_id, starts_at, ends_at, status \
                     FROM security_assignments ORDER BY starts_at",
                )
                .fetch_all(pool)
                .await?;
                Ok(rows
                    .into_iter()
                    .map(
                        |(id, agent_id, target, target_id, starts_at, ends_at, status)| {
                            SecurityAssignment {
                                id,
                                agent_id,
                                target: de(&target),
                                target_id,
                                starts_at,
                                ends_at,
                                status: de(&status),
                            }
                        },
                    )
                    .collect())
            }
            Store::Mem(m) => Ok(m
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .assignments
                .clone()),
        }
    }

    // -- warehouses -------------------------------------------------------

    pub async fn insert_warehouse(&self, w: &Warehouse) -> Result<(), sqlx::Error> {
        match self {
            Store::Pg(pool) => {
                sqlx::query(
                    "INSERT INTO warehouses (id, code, name, tier, latitude, longitude, geofence_radius_m) \
                     VALUES ($1, $2, $3, $4, $5, $6, $7) \
                     ON CONFLICT (id) DO UPDATE SET code = $2, name = $3, tier = $4, \
                       latitude = $5, longitude = $6, geofence_radius_m = $7",
                )
                .bind(&w.id)
                .bind(&w.code)
                .bind(&w.name)
                .bind(en(&w.tier))
                .bind(w.latitude)
                .bind(w.longitude)
                .bind(w.geofence_radius_m)
                .execute(pool)
                .await?;
                Ok(())
            }
            Store::Mem(m) => {
                m.lock()
                    .unwrap_or_else(|p| p.into_inner())
                    .warehouses
                    .insert(w.id.clone(), w.clone());
                Ok(())
            }
        }
    }

    pub async fn get_warehouse(&self, id: &str) -> Result<Option<Warehouse>, sqlx::Error> {
        match self {
            Store::Pg(pool) => {
                let row = sqlx::query_as::<_, (String, String, String, String, f64, f64, f64)>(
                    "SELECT id, code, name, tier, latitude, longitude, geofence_radius_m \
                     FROM warehouses WHERE id = $1",
                )
                .bind(id)
                .fetch_optional(pool)
                .await?;
                Ok(row.map(
                    |(id, code, name, tier, latitude, longitude, geofence_radius_m)| Warehouse {
                        id,
                        code,
                        name,
                        tier: de(&tier),
                        latitude,
                        longitude,
                        geofence_radius_m,
                    },
                ))
            }
            Store::Mem(m) => Ok(m
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .warehouses
                .get(id)
                .cloned()),
        }
    }

    pub async fn list_warehouses(&self) -> Result<Vec<Warehouse>, sqlx::Error> {
        match self {
            Store::Pg(pool) => {
                let rows = sqlx::query_as::<_, (String, String, String, String, f64, f64, f64)>(
                    "SELECT id, code, name, tier, latitude, longitude, geofence_radius_m \
                     FROM warehouses ORDER BY code",
                )
                .fetch_all(pool)
                .await?;
                Ok(rows
                    .into_iter()
                    .map(
                        |(id, code, name, tier, latitude, longitude, geofence_radius_m)| {
                            Warehouse {
                                id,
                                code,
                                name,
                                tier: de(&tier),
                                latitude,
                                longitude,
                                geofence_radius_m,
                            }
                        },
                    )
                    .collect())
            }
            Store::Mem(m) => Ok(m
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .warehouses
                .values()
                .cloned()
                .collect()),
        }
    }

    // -- transfers --------------------------------------------------------

    pub async fn insert_transfer(&self, t: &WarehouseTransfer) -> Result<(), sqlx::Error> {
        match self {
            Store::Pg(pool) => {
                sqlx::query(
                    "INSERT INTO warehouse_transfers (id, from_warehouse_id, to_warehouse_id, vehicle_id, \
                       status, expected_pallets, arrived_pallets, created_at, departed_at, arrived_at) \
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) \
                     ON CONFLICT (id) DO UPDATE SET from_warehouse_id = $2, to_warehouse_id = $3, \
                       vehicle_id = $4, status = $5, expected_pallets = $6, arrived_pallets = $7, \
                       created_at = $8, departed_at = $9, arrived_at = $10",
                )
                .bind(&t.id)
                .bind(&t.from_warehouse_id)
                .bind(&t.to_warehouse_id)
                .bind(&t.vehicle_id)
                .bind(en(&t.status))
                .bind(serde_json::to_value(&t.expected_pallets).unwrap_or(serde_json::json!([])))
                .bind(serde_json::to_value(&t.arrived_pallets).unwrap_or(serde_json::json!([])))
                .bind(t.created_at)
                .bind(t.departed_at)
                .bind(t.arrived_at)
                .execute(pool)
                .await?;
                Ok(())
            }
            Store::Mem(m) => {
                m.lock()
                    .unwrap_or_else(|p| p.into_inner())
                    .transfers
                    .insert(t.id.clone(), t.clone());
                Ok(())
            }
        }
    }

    pub async fn get_transfer(&self, id: &str) -> Result<Option<WarehouseTransfer>, sqlx::Error> {
        match self {
            Store::Pg(pool) => {
                let row = sqlx::query_as::<_, (String, String, String, Option<String>, String, serde_json::Value, serde_json::Value, DateTime<Utc>, Option<DateTime<Utc>>, Option<DateTime<Utc>>)>(
                    "SELECT id, from_warehouse_id, to_warehouse_id, vehicle_id, status, expected_pallets, \
                       arrived_pallets, created_at, departed_at, arrived_at FROM warehouse_transfers WHERE id = $1",
                )
                .bind(id)
                .fetch_optional(pool)
                .await?;
                Ok(row.map(
                    |(
                        id,
                        from_warehouse_id,
                        to_warehouse_id,
                        vehicle_id,
                        status,
                        expected_pallets,
                        arrived_pallets,
                        created_at,
                        departed_at,
                        arrived_at,
                    )| {
                        WarehouseTransfer {
                            id,
                            from_warehouse_id,
                            to_warehouse_id,
                            vehicle_id,
                            status: de(&status),
                            expected_pallets: serde_json::from_value(expected_pallets)
                                .unwrap_or_default(),
                            arrived_pallets: serde_json::from_value(arrived_pallets)
                                .unwrap_or_default(),
                            created_at,
                            departed_at,
                            arrived_at,
                        }
                    },
                ))
            }
            Store::Mem(m) => Ok(m
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .transfers
                .get(id)
                .cloned()),
        }
    }

    // -- pallets ----------------------------------------------------------

    pub async fn insert_pallet(&self, p: &Pallet) -> Result<(), sqlx::Error> {
        match self {
            Store::Pg(pool) => {
                let mut tx = pool.begin().await?;
                sqlx::query(
                    "INSERT INTO pallets (id, serial, warehouse_id, status, received_at, used_at_station) \
                     VALUES ($1, $2, $3, $4, $5, $6) \
                     ON CONFLICT (id) DO UPDATE SET serial = $2, warehouse_id = $3, status = $4, \
                       received_at = $5, used_at_station = $6",
                )
                .bind(&p.id)
                .bind(&p.serial)
                .bind(&p.warehouse_id)
                .bind(en(&p.status))
                .bind(p.received_at)
                .bind(&p.used_at_station)
                .execute(&mut *tx)
                .await?;
                sqlx::query("DELETE FROM pallet_items WHERE pallet_id = $1")
                    .bind(&p.id)
                    .execute(&mut *tx)
                    .await?;
                for item in &p.items {
                    sqlx::query(
                        "INSERT INTO pallet_items (id, pallet_id, item_type, reference_id, quantity) \
                         VALUES ($1, $2, $3, $4, $5)",
                    )
                    .bind(&item.id)
                    .bind(&p.id)
                    .bind(en(&item.item_type))
                    .bind(&item.reference_id)
                    .bind(item.quantity as i64)
                    .execute(&mut *tx)
                    .await?;
                }
                tx.commit().await?;
                Ok(())
            }
            Store::Mem(m) => {
                m.lock()
                    .unwrap_or_else(|p| p.into_inner())
                    .pallets
                    .insert(p.id.clone(), p.clone());
                Ok(())
            }
        }
    }

    pub async fn get_pallet(&self, id: &str) -> Result<Option<Pallet>, sqlx::Error> {
        match self {
            Store::Pg(pool) => {
                let row = sqlx::query_as::<
                    _,
                    (
                        String,
                        String,
                        String,
                        String,
                        DateTime<Utc>,
                        Option<String>,
                    ),
                >(
                    "SELECT id, serial, warehouse_id, status, received_at, used_at_station \
                     FROM pallets WHERE id = $1",
                )
                .bind(id)
                .fetch_optional(pool)
                .await?;
                let Some((id, serial, warehouse_id, status, received_at, used_at_station)) = row
                else {
                    return Ok(None);
                };
                let items = sqlx::query_as::<_, (String, String, String, i64)>(
                    "SELECT id, item_type, reference_id, quantity FROM pallet_items WHERE pallet_id = $1 ORDER BY id",
                )
                .bind(&id)
                .fetch_all(pool)
                .await?
                .into_iter()
                .map(|(id, item_type, reference_id, quantity)| warehouse::domain::PalletItem {
                    id,
                    item_type: de(&item_type),
                    reference_id,
                    quantity: quantity as u64,
                })
                .collect();
                Ok(Some(Pallet {
                    id,
                    serial,
                    warehouse_id,
                    status: de(&status),
                    items,
                    received_at,
                    used_at_station,
                }))
            }
            Store::Mem(m) => Ok(m
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .pallets
                .get(id)
                .cloned()),
        }
    }

    // -- incidents --------------------------------------------------------

    pub async fn insert_incident(&self, i: &Incident) -> Result<(), sqlx::Error> {
        match self {
            Store::Pg(pool) => {
                let mut tx = pool.begin().await?;
                sqlx::query(
                    "INSERT INTO incidents (id, warehouse_id, pallet_id, transfer_id, incident_type, \
                       severity, summary, status, reported_at, reported_by, resolved_at) \
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11) \
                     ON CONFLICT (id) DO UPDATE SET warehouse_id = $2, pallet_id = $3, transfer_id = $4, \
                       incident_type = $5, severity = $6, summary = $7, status = $8, reported_at = $9, \
                       reported_by = $10, resolved_at = $11",
                )
                .bind(&i.id)
                .bind(&i.warehouse_id)
                .bind(&i.pallet_id)
                .bind(&i.transfer_id)
                .bind(en(&i.incident_type))
                .bind(en(&i.severity))
                .bind(&i.summary)
                .bind(en(&i.status))
                .bind(i.reported_at)
                .bind(&i.reported_by)
                .bind(i.resolved_at)
                .execute(&mut *tx)
                .await?;
                sqlx::query("DELETE FROM incident_notes WHERE incident_id = $1")
                    .bind(&i.id)
                    .execute(&mut *tx)
                    .await?;
                for note in &i.notes {
                    sqlx::query(
                        "INSERT INTO incident_notes (id, incident_id, author, note, at) \
                         VALUES ($1, $2, $3, $4, $5)",
                    )
                    .bind(&note.id)
                    .bind(&i.id)
                    .bind(&note.author)
                    .bind(&note.note)
                    .bind(note.at)
                    .execute(&mut *tx)
                    .await?;
                }
                tx.commit().await?;
                Ok(())
            }
            Store::Mem(m) => {
                m.lock()
                    .unwrap_or_else(|p| p.into_inner())
                    .incidents
                    .insert(i.id.clone(), i.clone());
                Ok(())
            }
        }
    }

    pub async fn get_incident(&self, id: &str) -> Result<Option<Incident>, sqlx::Error> {
        match self {
            Store::Pg(pool) => {
                let row = sqlx::query_as::<_, (String, Option<String>, Option<String>, Option<String>, String, String, String, String, DateTime<Utc>, String, Option<DateTime<Utc>>)>(
                    "SELECT id, warehouse_id, pallet_id, transfer_id, incident_type, severity, summary, \
                       status, reported_at, reported_by, resolved_at FROM incidents WHERE id = $1",
                )
                .bind(id)
                .fetch_optional(pool)
                .await?;
                let Some((
                    id,
                    warehouse_id,
                    pallet_id,
                    transfer_id,
                    incident_type,
                    severity,
                    summary,
                    status,
                    reported_at,
                    reported_by,
                    resolved_at,
                )) = row
                else {
                    return Ok(None);
                };
                let notes = sqlx::query_as::<_, (String, String, String, DateTime<Utc>)>(
                    "SELECT id, author, note, at FROM incident_notes WHERE incident_id = $1 ORDER BY at",
                )
                .bind(&id)
                .fetch_all(pool)
                .await?
                .into_iter()
                .map(|(id, author, note, at)| warehouse::domain::IncidentNote { id, author, note, at })
                .collect();
                Ok(Some(Incident {
                    id,
                    warehouse_id,
                    pallet_id,
                    transfer_id,
                    incident_type: de(&incident_type),
                    severity: de(&severity),
                    summary,
                    status: de(&status),
                    reported_at,
                    reported_by,
                    resolved_at,
                    notes,
                }))
            }
            Store::Mem(m) => Ok(m
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .incidents
                .get(id)
                .cloned()),
        }
    }

    pub async fn list_incidents(&self) -> Result<Vec<Incident>, sqlx::Error> {
        match self {
            Store::Pg(pool) => {
                let rows = sqlx::query_as::<_, (String, Option<String>, Option<String>, Option<String>, String, String, String, String, DateTime<Utc>, String, Option<DateTime<Utc>>)>(
                    "SELECT id, warehouse_id, pallet_id, transfer_id, incident_type, severity, summary, \
                       status, reported_at, reported_by, resolved_at FROM incidents ORDER BY reported_at DESC",
                )
                .fetch_all(pool)
                .await?;
                let mut out = Vec::with_capacity(rows.len());
                for (
                    id,
                    warehouse_id,
                    pallet_id,
                    transfer_id,
                    incident_type,
                    severity,
                    summary,
                    status,
                    reported_at,
                    reported_by,
                    resolved_at,
                ) in rows
                {
                    let notes = sqlx::query_as::<_, (String, String, String, DateTime<Utc>)>(
                        "SELECT id, author, note, at FROM incident_notes WHERE incident_id = $1 ORDER BY at",
                    )
                    .bind(&id)
                    .fetch_all(pool)
                    .await?
                    .into_iter()
                    .map(|(id, author, note, at)| warehouse::domain::IncidentNote { id, author, note, at })
                    .collect();
                    out.push(Incident {
                        id,
                        warehouse_id,
                        pallet_id,
                        transfer_id,
                        incident_type: de(&incident_type),
                        severity: de(&severity),
                        summary,
                        status: de(&status),
                        reported_at,
                        reported_by,
                        resolved_at,
                        notes,
                    });
                }
                Ok(out)
            }
            Store::Mem(m) => Ok(m
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .incidents
                .values()
                .cloned()
                .collect()),
        }
    }

    // -- assets -----------------------------------------------------------

    pub async fn insert_asset(&self, a: &Asset) -> Result<(), sqlx::Error> {
        match self {
            Store::Pg(pool) => {
                sqlx::query(
                    "INSERT INTO assets (asset_id, asset_type, serial, model, status, station_code, last_seen_at) \
                     VALUES ($1, $2, $3, $4, $5, $6, $7) \
                     ON CONFLICT (asset_id) DO UPDATE SET asset_type = $2, serial = $3, model = $4, \
                       status = $5, station_code = $6, last_seen_at = $7",
                )
                .bind(&a.asset_id)
                .bind(en(&a.asset_type))
                .bind(&a.serial)
                .bind(&a.model)
                .bind(en(&a.status))
                .bind(&a.station_code)
                .bind(a.last_seen_at)
                .execute(pool)
                .await?;
                Ok(())
            }
            Store::Mem(m) => {
                m.lock()
                    .unwrap_or_else(|p| p.into_inner())
                    .assets
                    .insert(a.asset_id.clone(), a.clone());
                Ok(())
            }
        }
    }

    pub async fn get_asset(&self, id: &str) -> Result<Option<Asset>, sqlx::Error> {
        match self {
            Store::Pg(pool) => {
                let row = sqlx::query_as::<_, (String, String, String, String, String, Option<String>, DateTime<Utc>)>(
                    "SELECT asset_id, asset_type, serial, model, status, station_code, last_seen_at \
                     FROM assets WHERE asset_id = $1",
                )
                .bind(id)
                .fetch_optional(pool)
                .await?;
                Ok(row.map(
                    |(asset_id, asset_type, serial, model, status, station_code, last_seen_at)| {
                        Asset {
                            asset_id,
                            asset_type: de(&asset_type),
                            serial,
                            model,
                            status: de(&status),
                            station_code,
                            last_seen_at,
                        }
                    },
                ))
            }
            Store::Mem(m) => Ok(m
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .assets
                .get(id)
                .cloned()),
        }
    }

    pub async fn list_assets(&self) -> Result<Vec<Asset>, sqlx::Error> {
        match self {
            Store::Pg(pool) => {
                let rows = sqlx::query_as::<_, (String, String, String, String, String, Option<String>, DateTime<Utc>)>(
                    "SELECT asset_id, asset_type, serial, model, status, station_code, last_seen_at \
                     FROM assets ORDER BY asset_id",
                )
                .fetch_all(pool)
                .await?;
                Ok(rows
                    .into_iter()
                    .map(
                        |(
                            asset_id,
                            asset_type,
                            serial,
                            model,
                            status,
                            station_code,
                            last_seen_at,
                        )| Asset {
                            asset_id,
                            asset_type: de(&asset_type),
                            serial,
                            model,
                            status: de(&status),
                            station_code,
                            last_seen_at,
                        },
                    )
                    .collect())
            }
            Store::Mem(m) => Ok(m
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .assets
                .values()
                .cloned()
                .collect()),
        }
    }

    // -- custody events ---------------------------------------------------

    pub async fn insert_custody_event(&self, e: &CustodyEvent) -> Result<(), sqlx::Error> {
        match self {
            Store::Pg(pool) => {
                sqlx::query(
                    "INSERT INTO custody_events (event_id, asset_id, from_custodian, to_custodian, \
                       location, ts_hlc, signature, seq) \
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
                )
                .bind(&e.event_id)
                .bind(&e.asset_id)
                .bind(&e.from_custodian)
                .bind(&e.to_custodian)
                .bind(&e.location)
                .bind(e.ts_hlc)
                .bind(&e.signature)
                .bind(e.seq as i64)
                .execute(pool)
                .await?;
                Ok(())
            }
            Store::Mem(m) => {
                m.lock()
                    .unwrap_or_else(|p| p.into_inner())
                    .custody_events
                    .push(e.clone());
                Ok(())
            }
        }
    }

    pub async fn custody_history(&self, asset_id: &str) -> Result<Vec<CustodyEvent>, sqlx::Error> {
        match self {
            Store::Pg(pool) => {
                let rows = sqlx::query_as::<_, (String, String, String, String, String, i64, String, i64)>(
                    "SELECT event_id, asset_id, from_custodian, to_custodian, location, ts_hlc, signature, seq \
                     FROM custody_events WHERE asset_id = $1 ORDER BY seq",
                )
                .bind(asset_id)
                .fetch_all(pool)
                .await?;
                Ok(rows
                    .into_iter()
                    .map(
                        |(
                            event_id,
                            asset_id,
                            from_custodian,
                            to_custodian,
                            location,
                            ts_hlc,
                            signature,
                            seq,
                        )| CustodyEvent {
                            event_id,
                            asset_id,
                            from_custodian,
                            to_custodian,
                            location,
                            ts_hlc,
                            signature,
                            seq: seq as u64,
                        },
                    )
                    .collect())
            }
            Store::Mem(m) => Ok(m
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .custody_events
                .iter()
                .filter(|e| e.asset_id == asset_id)
                .cloned()
                .collect()),
        }
    }

    // -- shortfalls (inventory/management.yaml) -------------------------------

    pub async fn insert_shortfall(
        &self,
        shortfall: &crate::spec::Shortfall,
    ) -> Result<(), sqlx::Error> {
        match self {
            Store::Pg(pool) => {
                sqlx::query(
                    "INSERT INTO inventory_shortfalls (shortfall_id, constituency_code, item, quantity, reported_by_hash, status, created_at, request_json) \
                     VALUES ($1,$2,$3,$4,$5,$6,$7,$8) \
                     ON CONFLICT (shortfall_id) DO NOTHING",
                )
                .bind(&shortfall.shortfall_id)
                .bind(&shortfall.request.constituency_code)
                .bind(&shortfall.request.item)
                .bind(shortfall.request.quantity as i32)
                .bind(&shortfall.request.reported_by_hash)
                .bind(&shortfall.status)
                .bind(
                    shortfall
                        .created_at
                        .parse::<DateTime<Utc>>()
                        .unwrap_or_else(|_| Utc::now()),
                )
                .bind(serde_json::to_value(&shortfall.request).unwrap_or(serde_json::json!({})))
                .execute(pool)
                .await?;
                Ok(())
            }
            Store::Mem(m) => {
                m.lock()
                    .unwrap_or_else(|p| p.into_inner())
                    .shortfalls
                    .insert(shortfall.shortfall_id.clone(), shortfall.clone());
                Ok(())
            }
        }
    }

    pub async fn list_shortfalls(&self) -> Result<Vec<crate::spec::Shortfall>, sqlx::Error> {
        match self {
            Store::Pg(pool) => {
                let rows = sqlx::query_as::<_, (String, String, String, i32, String, String, DateTime<Utc>, serde_json::Value)>(
                    "SELECT shortfall_id, constituency_code, item, quantity, reported_by_hash, status, created_at, request_json \
                     FROM inventory_shortfalls ORDER BY created_at",
                )
                .fetch_all(pool)
                .await?;
                Ok(rows
                    .into_iter()
                    .map(
                        |(
                            shortfall_id,
                            constituency_code,
                            item,
                            quantity,
                            reported_by_hash,
                            status,
                            created_at,
                            request_json,
                        )| {
                            let request: crate::spec::ShortfallRequest =
                                serde_json::from_value(request_json).unwrap_or(crate::spec::ShortfallRequest {
                                    constituency_code: constituency_code.clone(),
                                    item: item.clone(),
                                    quantity: quantity as u32,
                                    reported_by_hash: reported_by_hash.clone(),
                                });
                            crate::spec::Shortfall {
                                shortfall_id,
                                status,
                                created_at: created_at.to_rfc3339(),
                                request,
                            }
                        },
                    )
                    .collect())
            }
            Store::Mem(m) => Ok(m
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .shortfalls
                .values()
                .cloned()
                .collect()),
        }
    }
}
