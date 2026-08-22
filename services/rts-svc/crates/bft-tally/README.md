# eel-rts-bft-tally

HotStuff BFT consensus for tally aggregation (HARD-B item 8).

## Algorithm

Implements the linear HotStuff protocol:

> Yin, M., Malkhi, D., Reiter, M. K., Gueta, G. G., Abraham, I.
> "HotStuff: BFT Consensus with Linearity and Responsiveness."
> PODC 2019. ACM, pp. 347-356.

Configuration: N=4 replicas, f=1 Byzantine fault tolerance, quorum=3 (2f+1).

Four phases per view: Prepare, PreCommit, Commit, Decide. Each phase requires
a QuorumCertificate (QC) from the previous phase. Linear communication: leader
collects votes, broadcasts QC; O(n) messages per phase.

## Comparison: Kafka-transactional vs HotStuff BFT

| Property            | Kafka-transactional         | HotStuff BFT                        |
|---------------------|-----------------------------|-------------------------------------|
| Safety model        | Broker-level crash fault    | Byzantine (up to f faulty replicas) |
| Liveness            | Kafka broker available      | f+1 correct replicas                |
| Latency (happy path)| 1 Kafka round-trip          | 3 message delays                    |
| Throughput overhead | Low                         | 3x vote messages per block          |
| Leader failure      | Consumer restart             | View change (pacemaker)             |
| Rewrite attack      | Possible at broker           | Cryptographically impossible (Ed25519 QC) |
| Audit trail         | Kafka log                    | QC chain (each block requires threshold sig) |

## Performance

Latency per block: 3 * network_round_trip (happy path, no view change).
For a 10ms inter-datacenter round-trip: ~30ms per tally block.
Reference: HotStuff paper Section 7, Table 2.

## Security

Ed25519 signatures (RFC 8032) on (msg_type, view_number, block_hash).
A valid QC requires QUORUM=3 distinct replica signatures. An adversary
controlling f=1 replica cannot forge a QC or cause safety violations.
