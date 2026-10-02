# SA-003: Kenyan-sovereign edge: multi-ISP peering vs Cloudflare / Akamai

Date: 2026-08-21
Applies to: Datacraft bid documents under work_docs/01-Projects/the Commission/sw/ (our own earlier draft)
Status: decided (ADR takes precedence; sw/ docs to be edited)

## Earlier draft (sw/) says

`sw/ARCHITECTURE.md` §3.1 / §12 names **Cloudflare Magic Transit or Akamai Prolexic** for DDoS mitigation and CDN.

## Decision (per ADRs)

- Multi-ISP peering + upstream scrubbing agreements with **Kenyan ISPs** (Safaricom, Airtel, Jamii, iColo, Konza).
- Multi-provider anycast + BGP inside Kenya.
- **Coraza WAF + Anubis PoW** at each site's Envoy Gateway ingress.
- Origin shield with nginx + Varnish at each of the 2 DR sites + 5 regional edges (ADR-0019).

## RFP status

**Not RFP-binding directly**, tender.txt does not name a specific edge/CDN provider. **However, the spec is internally self-contradictory**: `sw/ARCHITECTURE.md` §17.2 mandates Kenya-sovereign residency for public data publication, which §3.1's Cloudflare/Akamai proposal violates. This CR reconciles the internal contradiction against the sovereign posture.

## Rationale

1. **Sovereignty**. Kenya-national-election critical infrastructure must sit on Kenyan-controlled edges. Cloudflare and Akamai are US corporations subject to US jurisdiction; a national-election traffic firehose flowing through them is a sovereignty compromise.
2. **Rug-pull risk**. Cloudflare has a documented pattern of dropping controversial customers (Kiwi Farms, 8chan) after public pressure. On an election with disinformation-campaign risk, the possibility of Cloudflare unilaterally withdrawing service during the election window is a live threat.
3. **Legal-process exposure**. Both Cloudflare and Akamai are subject to US legal process (FISA, national security letters) that could compel logging or route-manipulation without our knowledge.
4. **Bandwidth economics**. Cloudflare and Akamai contracts are USD-denominated multi-year. Kenyan ISP peering leverages existing Kenyan bandwidth infrastructure at KES cost.
5. **The spec already knows this**. `sw/ARCHITECTURE.md` §17.2 mandates IPFS-in-Kenya, Kenya Gazette print anchoring, and KBC Radio hash bulletin publication. The §3.1 Cloudflare/Akamai proposal is a stale artefact of a pre-sovereignty draft.

## Trade-offs

- Kenyan ISP capacity + scrubbing sophistication is less than Cloudflare/Akamai's global scale. Real. Mitigated by multi-ISP peering + BGP flowspec agreements + the 5-regional-edge topology (ADR-0019) reducing single-target exposure.
- Kenyan-only DDoS defence means a state-actor-scale attack from outside Kenya lands entirely on Kenyan telco infrastructure; scrubbing agreements must be formalised pre-freeze.
- Setup cost of ISP peering agreements is real engineering + relationship work; mitigated by starting T-9 months.

## Edits to apply to sw/

If Cloudflare/Akamai is politically mandated: enforce **request-path isolation**, Cloudflare handles only anonymous public read traffic to the signed-snapshot tier (ADR-0011 Tier 1). All authenticated traffic (Tiers 2, 3, 5), all administrative surfaces, and all voter-facing personal-lookup endpoints route directly through Kenyan-ISP edges bypassing the foreign CDN. Consensus data verifiability (cosign + Merkle inclusion proofs) means the foreign CDN cannot tamper with published results, but observability of who requested what leaks to the foreign provider.

## Related

ADR-0010 (edge & DDoS), ADR-0019 (regional edge topology), ADR-0017 (transparency publication channels), ADR-0023 (RFP-alignment).
