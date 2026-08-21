use eel_eventstore_rs::StoredEvent;
use rts_domain::aggregates::form34a::CandidateResult;
use rts_domain::aggregates::form34b::{Form34B, Form34BStatus};
use rts_domain::aggregates::form34c::{Form34C, Form34CStatus};
use uuid::Uuid;

pub fn aggregate_34b(constituency: &str, form34a_events: Vec<StoredEvent>) -> Form34B {
    let stream_id = Uuid::new_v4();
    let mut form34b = Form34B::new(stream_id);
    form34b.constituency_code = constituency.to_string();
    for event in &form34a_events {
        let sid = event.stream_id.to_string();
        if !form34b.compiled_stations.contains(&sid) { form34b.compiled_stations.push(sid); }
        if let Some(v) = event.payload.get("registered_voters").and_then(|v| v.as_u64()) { form34b.total_registered += v as u32; }
        if let Some(v) = event.payload.get("valid_votes").and_then(|v| v.as_u64()) { form34b.total_valid += v as u32; }
        if let Some(v) = event.payload.get("rejected_votes").and_then(|v| v.as_u64()) { form34b.total_rejected += v as u32; }
        if let Some(v) = event.payload.get("spoilt_ballots").and_then(|v| v.as_u64()) { form34b.total_spoilt += v as u32; }
        if let Some(candidates) = event.payload.get("candidates").and_then(|c| c.as_array()) {
            for candidate in candidates {
                let cid = candidate.get("candidate_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let cname = candidate.get("candidate_name").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let votes = candidate.get("votes").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                if let Some(ex) = form34b.candidate_totals.iter_mut().find(|c| c.candidate_id == cid) {
                    ex.votes += votes;
                } else {
                    form34b.candidate_totals.push(CandidateResult { candidate_id: cid, candidate_name: cname, votes });
                }
            }
        }
    }
    form34b.status = Form34BStatus::Compiled;
    form34b.version = form34a_events.len() as i64;
    form34b
}

pub fn aggregate_34c(form34b_list: Vec<Form34B>) -> Form34C {
    let stream_id = Uuid::new_v4();
    let mut form34c = Form34C::new(stream_id);
    for form34b in &form34b_list {
        if !form34c.compiled_constituencies.contains(&form34b.constituency_code) {
            form34c.compiled_constituencies.push(form34b.constituency_code.clone());
        }
        form34c.total_registered += form34b.total_registered as u64;
        form34c.total_valid += form34b.total_valid as u64;
        form34c.total_rejected += form34b.total_rejected as u64;
        form34c.total_spoilt += form34b.total_spoilt as u64;
        for c in &form34b.candidate_totals {
            if let Some(ex) = form34c.candidate_totals.iter_mut().find(|x| x.candidate_id == c.candidate_id) {
                ex.votes += c.votes;
            } else {
                form34c.candidate_totals.push(c.clone());
            }
        }
    }
    form34c.status = Form34CStatus::Compiled;
    form34c.version = form34b_list.len() as i64;
    form34c
}
