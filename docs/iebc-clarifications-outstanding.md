# IEBC clarifications outstanding

Questions only IEBC can answer. None of these are stack decisions; the stack is fixed by the ADRs. Raised via the pre-bid clarification channel.

### 2.1 Voter register retention contradiction (5y vs 15y)
- **Impact**: DATA-MODEL.md says "contract + 5 years" while Elections Act § and expect.md require 15 years. Choosing wrong triggers either statutory non-compliance or over-retention DPA exposure.
- **Ask**: IEBC counsel confirms authoritative retention window for voter register (M02, M35). Update DATA-MODEL.md.
- **Also flagged in**: `sw/INTEGRATION-AUDIT.md` Blocker 1.

### 2.2 Digital signature + QR standards for Form 34A (PRE_BID C13)
- **Impact**: cannot spec the M08 RTS device signature scheme without IEBC's chosen algorithm / key-length.
- **Ask**: IEBC answer PRE_BID C13 with a specific signature suite (recommended: Ed25519 or ECDSA P-384 aligned with ADR-0005 hybrid stack).

### 2.3 Log retention specifics (PRE_BID D13)
- **Impact**: our ADR-0015 amendment aligned to 90-day online per RFP but the "permanent archive" details need confirming (medium, format, per-tier retention policy, deletion/erasure interaction).
- **Ask**: IEBC clarifies PRE_BID D13.

### 2.4 ODPC registration timing (PRE_BID D10)
- **Impact**: bidder needs to know whether ODPC data-controller registration must be in place at bid, at award, or before commercial operation.
- **Ask**: IEBC answers PRE_BID D10.

### 2.5 Kenya data residency for production data (PRE_BID D9)
- **Impact**: if IEBC mandates full Kenya-only residency for production data (not just publication), no offshore support access is possible. Affects Tier 1-3 support desk model, remote-hire mobility, and any offshore build / test / staging.
- **Ask**: IEBC answers PRE_BID D9.

### 4.2 Independent security certifier (PRE_BID D11)
- Elections (Technology) Regulations 2017 require an independent security certifier. IEBC has not named it.
- **Ask**: IEBC answers PRE_BID D11. Certifier's requirements may add compliance obligations we should bake into ADR-0009 / ADR-0022.

### 4.3 Bidder-provided vs IEBC-provided DC hardware (PRE_BID C1)
- If IEBC provides all DC hardware, our K8s cluster provisioning story is entirely on IEBC-owned tin. Deployment automation and hardware-selector work depends on this.
- **Ask**: IEBC clarifies PRE_BID C1.

---

Owner (Datacraft-side): TBD.
Owner (IEBC-side): TBD.
Next sync: TBD.

Owner (Datacraft-side): TBD. Next IEBC sync: TBD.
