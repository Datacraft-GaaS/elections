use serde_json::{json, Value};
use uuid::Uuid;

const SOURCE: &str = "spiffe://eel.local/service/portal-inspection";

pub fn roll_snapshotted_v1(snapshot_id: &Uuid, snapshot_hash: &str) -> Value {
    json!({
        "specversion": "1.0",
        "type": "eel.portal.roll_snapshotted.v1",
        "source": SOURCE,
        "id": Uuid::now_v7().to_string(),
        "subject": format!("roll-snapshot/{}", snapshot_id),
        "datacontenttype": "application/json",
        "data": {
            "snapshot_id": snapshot_id.to_string(),
            "snapshot_hash": snapshot_hash,
        }
    })
}

pub fn objection_filed_v1(objection_id: &Uuid, station_id: &str) -> Value {
    json!({
        "specversion": "1.0",
        "type": "eel.portal.objection_filed.v1",
        "source": SOURCE,
        "id": Uuid::now_v7().to_string(),
        "subject": format!("objection/{}", objection_id),
        "datacontenttype": "application/json",
        "data": {
            "objection_id": objection_id.to_string(),
            "station_id": station_id,
        }
    })
}

pub fn correction_referred_v1(claim_id: &Uuid, ecourt_ref: &str) -> Value {
    json!({
        "specversion": "1.0",
        "type": "eel.portal.correction_referred.v1",
        "source": SOURCE,
        "id": Uuid::now_v7().to_string(),
        "subject": format!("correction-claim/{}", claim_id),
        "datacontenttype": "application/json",
        "data": {
            "claim_id": claim_id.to_string(),
            "ecourt_ref": ecourt_ref,
        }
    })
}
