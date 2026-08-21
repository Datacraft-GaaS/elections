# eel-abis-smpc-match

Secure Multi-Party Computation biometric matching for the eel ABIS stack.

## Modules

| Module | Function |
|---|---|
| `protocol` | Protocol enum (SPDZ/BGW/GMW/Yao/AB+) + party-count validation |
| `session` | SmpcSession lifecycle: create/abort/complete + eel_smpc_sessions logging |
| `share` | Shamir secret sharing over Fp (61-bit Mersenne prime) |
| `match_dot` | Secure dot-product under SPDZ (3-party local simulation) |
| `policy` | Minimum threshold, party diversity, adversary-bound enforcement |

## Security Model

The 3-party simulation ensures no single party learns the full input vectors.
In production, parties run on isolated infrastructure; abort semantics detect
any party deviating from the protocol.

## Tests

Run with: `cargo test -p eel-abis-smpc-match`

20+ tests covering reconstruction correctness, abort detection, threshold enforcement.
