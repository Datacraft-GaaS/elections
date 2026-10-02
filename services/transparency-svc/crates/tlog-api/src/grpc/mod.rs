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

//! gRPC service implementation using tonic.

pub mod transparency {
    tonic::include_proto!("eel.transparency.v1");
}

use tlog_domain::Domain;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};
use transparency::transparency_log_server::{TransparencyLog, TransparencyLogServer};
use transparency::*;

use crate::state::ApiState;

pub struct TlogGrpcService {
    pub state: ApiState,
}

#[tonic::async_trait]
impl TransparencyLog for TlogGrpcService {
    type SubscribeSthStream = ReceiverStream<Result<SignedTreeHeadResponse, Status>>;

    async fn append_leaf(
        &self,
        request: Request<LeafRequest>,
    ) -> Result<Response<InclusionProofResponse>, Status> {
        let req = request.into_inner();
        let domain: Domain = req
            .domain
            .parse()
            .map_err(|_| Status::invalid_argument("invalid domain"))?;
        let event_time = chrono::DateTime::from_timestamp_micros(req.event_time_us)
            .ok_or_else(|| Status::invalid_argument("invalid event_time_us"))?;

        let result = self
            .state
            .sequencer
            .append(
                domain,
                &req.canonical_event_json,
                &req.producer_spiffe_id,
                &req.source_event_id,
                &req.source_topic,
                event_time,
                &req.producer_signature,
            )
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        match result {
            None => Ok(Response::new(InclusionProofResponse {
                leaf_index: 0,
                tree_size: 0,
                root_hash: String::new(),
                hashes: vec![],
                rate_limited: true,
            })),
            Some(r) => Ok(Response::new(InclusionProofResponse {
                leaf_index: r.leaf_index,
                tree_size: r.tree_size,
                root_hash: r.proof.root_hash,
                hashes: r.proof.hashes,
                rate_limited: false,
            })),
        }
    }

    async fn get_proof(
        &self,
        request: Request<ProofQuery>,
    ) -> Result<Response<ProofResponse>, Status> {
        let query = request.into_inner().query.ok_or_else(|| {
            Status::invalid_argument("query must be inclusion or consistency")
        })?;
        match query {
            transparency::proof_query::Query::Inclusion(q) => {
                let leaf_hash = hex::decode(&q.leaf_hash)
                    .map_err(|_| Status::invalid_argument("leaf_hash must be hex"))?;
                if leaf_hash.len() != 32 {
                    return Err(Status::invalid_argument("leaf_hash must be 32 bytes"));
                }
                let proof = tlog_sequencer::proofs::inclusion_proof(&self.state.pool, &leaf_hash)
                    .await
                    .map_err(|e| Status::internal(e.to_string()))?;
                let Some(proof) = proof else {
                    return Err(Status::not_found("unknown leaf hash"));
                };
                Ok(Response::new(ProofResponse {
                    proof: Some(transparency::proof_response::Proof::Inclusion(
                        InclusionProofResponse {
                            leaf_index: proof.leaf_index,
                            tree_size: proof.tree_size,
                            root_hash: proof.root_hash,
                            hashes: proof.hashes,
                            rate_limited: false,
                        },
                    )),
                }))
            }
            transparency::proof_query::Query::Consistency(q) => {
                if q.from_size == 0 || q.from_size > q.to_size {
                    return Err(Status::invalid_argument(
                        "from_size must be non-zero and <= to_size",
                    ));
                }
                let proof =
                    tlog_sequencer::proofs::consistency_proof(&self.state.pool, q.from_size)
                        .await
                        .map_err(|e| Status::internal(e.to_string()))?;
                let Some(proof) = proof else {
                    return Err(Status::not_found("tree smaller than from_size"));
                };
                if proof.new_size < q.to_size {
                    return Err(Status::out_of_range("to_size beyond current tree size"));
                }
                Ok(Response::new(ProofResponse {
                    proof: Some(transparency::proof_response::Proof::Consistency(
                        ConsistencyProofResponse {
                            old_size: proof.old_size,
                            new_size: proof.new_size,
                            old_root: proof.old_root,
                            new_root: proof.new_root,
                            hashes: proof.hashes,
                        },
                    )),
                }))
            }
        }
    }

    async fn get_sth(
        &self,
        request: Request<SthQuery>,
    ) -> Result<Response<SignedTreeHeadResponse>, Status> {
        let q = request.into_inner();
        sth_row(&self.state.pool, q.tree_size, q.cadence.as_deref())
            .await?
            .ok_or_else(|| Status::not_found("no signed tree head found"))
            .map(|sth| Response::new(sth))
    }

    async fn subscribe_sth(
        &self,
        request: Request<SthQuery>,
    ) -> Result<Response<Self::SubscribeSthStream>, Status> {
        let q = request.into_inner();
        let pool = self.state.pool.clone();
        let (tx, rx) = tokio::sync::mpsc::channel::<Result<SignedTreeHeadResponse, Status>>(16);

        tokio::spawn(async move {
            // Poll for new STHs at the hot cadence; push each unseen head.
            let mut ticker = tokio::time::interval(std::time::Duration::from_secs(
                tlog_domain::SthCadence::Hot.interval_secs(),
            ));
            let mut last_id = String::new();
            loop {
                ticker.tick().await;
                let next = sth_row(&pool, q.tree_size, q.cadence.as_deref()).await;
                match next {
                    Ok(Some(sth)) if sth.id != last_id => {
                        last_id = sth.id.clone();
                        if tx.send(Ok(sth)).await.is_err() {
                            break;
                        }
                    }
                    Ok(_) => {}
                    Err(status) => {
                        let _ = tx.send(Err(status)).await;
                        break;
                    }
                }
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}

pub fn service(state: ApiState) -> TransparencyLogServer<TlogGrpcService> {
    TransparencyLogServer::new(TlogGrpcService { state })
}

/// Fetch the newest SHT matching optional `tree_size` / `cadence` filters.
async fn sth_row(
    pool: &sqlx::PgPool,
    tree_size: Option<u64>,
    cadence: Option<&str>,
) -> Result<Option<SignedTreeHeadResponse>, Status> {
    let mut sql = String::from(
        "SELECT id::text, tree_size, root_hash, domain, cadence, signed_at, signature, \
         key_role, key_fingerprint FROM sth",
    );
    let mut clauses: Vec<String> = Vec::new();
    if tree_size.is_some() {
        clauses.push("tree_size = $1".to_owned());
    }
    if cadence.is_some() {
        clauses.push(format!("cadence = ${}", clauses.len() + 1));
    }
    if !clauses.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&clauses.join(" AND "));
    }
    sql.push_str(" ORDER BY tree_size DESC, signed_at DESC LIMIT 1");

    let mut q = sqlx::query_as::<
        _,
        (
            String,
            i64,
            Vec<u8>,
            Option<String>,
            String,
            chrono::DateTime<chrono::Utc>,
            Vec<u8>,
            String,
            String,
        ),
    >(&sql);
    if let Some(ts) = tree_size {
        q = q.bind(ts as i64);
    }
    if let Some(c) = cadence {
        q = q.bind(c);
    }

    let row = q
        .fetch_optional(pool)
        .await
        .map_err(|e| Status::internal(e.to_string()))?;
    Ok(row.map(
        |(id, ts, root_hash, domain, cadence, signed_at, sig, key_role, key_fp)| {
            SignedTreeHeadResponse {
                id,
                tree_size: ts as u64,
                root_hash: hex::encode(root_hash),
                cadence,
                signed_at_us: signed_at.timestamp_micros(),
                signature: hex::encode(sig),
                key_role,
                key_fingerprint: key_fp,
                domain,
            }
        },
    ))
}
