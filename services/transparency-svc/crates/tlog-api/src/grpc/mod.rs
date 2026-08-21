//! gRPC service implementation using tonic.

pub mod transparency {
    tonic::include_proto!("eel.transparency.v1");
}

use std::sync::Arc;

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
        _request: Request<ProofQuery>,
    ) -> Result<Response<ProofResponse>, Status> {
        Err(Status::unimplemented("proof queries not yet implemented"))
    }

    async fn get_sth(
        &self,
        _request: Request<SthQuery>,
    ) -> Result<Response<SignedTreeHeadResponse>, Status> {
        Err(Status::unimplemented("get_sth not yet implemented"))
    }

    async fn subscribe_sth(
        &self,
        _request: Request<SthQuery>,
    ) -> Result<Response<Self::SubscribeSthStream>, Status> {
        let (tx, rx) = tokio::sync::mpsc::channel(16);
        drop(tx);
        Ok(Response::new(ReceiverStream::new(rx)))
    }
}

pub fn service(state: ApiState) -> TransparencyLogServer<TlogGrpcService> {
    TransparencyLogServer::new(TlogGrpcService { state })
}
