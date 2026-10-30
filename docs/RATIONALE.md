# Why eel exists

This is the rationale for the work in this repository: what problem it solves, the
evidence that the problem is real, and the specific decisions that follow from
taking it seriously.

It is written to be read by engineers evaluating whether to trust, adopt, or
contribute to this code — and by anyone deciding whether an election system
should be built this way at all.

---

## The problem

Election systems fail in ways that are invisible until they matter, and the
failure modes are not primarily technical. They are procedural, legal, and
about trust.

Across the three most recent *Kenyan* general election cycles, the recurring
grounds of electoral petition tell a consistent story:

- **Results transmission broke, and the gap was discovered by the public.**
  In 2013 the results system crashed on election night; roughly half of
  polling stations never transmitted. The failure was not hidden, but it was
  not detected, predicted, or communicated in advance either.
- **Result forms did not reconcile.** In 2017, Form 34A station returns did
  not match the Form 34B aggregate at the national tallying centre. The
  Supreme Court held the tallying process "was not transparent, verifiable, or
  accurate" — and this ground was dispositive. The election was annulled.
- **The record could not be produced when a court asked for it.** Also in
  2017, the Commission declined the Supreme Court's request for server and log
  access. The Court drew an adverse inference, and the refusal itself became
  an annulment ground. This is the most instructive failure in the set: the
  system did not have to be wrong. It only had to be unable to show its work.
- **Supplier nationality became a ground of challenge.** In 2022, vendor
  staff nationality and admin-level access were argued as grounds. Dismissed
  on the facts that year. The same argument would be available next time if
  the access model has not changed.
- **Assets went unaccounted for.** A 2024 Auditor-General report found
  hundreds of biometric kits unaccounted for after an election.

The pattern: each failure is individually explicable, and the recurring theme
is that **the system could not produce evidence of its own correctness at the
moment correctness was contested.**

Confidence in the electoral authority follows the same curve. Afrobarometer
tracking puts trust below 50% after each disputed cycle, with the sharpest
drops among younger and urban voters. Three narratives do most of the damage,
and all three are about opacity rather than arithmetic:

1. *They hid the servers.*
2. *They falsified the forms.*
3. *Foreigners run the system.*

Each is an argument about what cannot be seen. Each becomes unanswerable when
the only response is "trust us."

---

## The constraint that shapes everything

An election system cannot be made trustworthy by being well-built. It is
trusted when a sceptic — one with no relationship to the operator and a
reason to be hostile — can verify its claims independently and reach the same
answer.

That reframes the goal. The deliverable is not a system that computes results
correctly. It is a system that **demonstrates** that it computed results
correctly, to anyone, without permission.

Everything below follows from that.

---

## The decisions

### 1. One Merkle log per election, published continuously

A single append-only, hash-chained transparency log per election. Signed tree
heads at 1-second, 60-second, and 5-minute cadences, HSM-signed, published to
five independent channels including daily print anchoring and national radio.

*Why:* a single log means there is one ordering, so there is nothing to
reconcile. Publishing to five channels means suppressing the log requires
collusion across media that do not share a failure mode — and a root read
aloud on radio cannot be quietly edited.

*What it defeats:* the "root was changed after publication" claim, and the
suppression of the root itself.

### 2. Reproducibility as a product feature, not an audit

Given the published inputs, any observer re-derives every intermediate tally
and the final result, and reaches bit-identical answers. Aggregation is pure
integer arithmetic. Timestamps are explicit inputs — no wall-clock reads, no
external state reads, so nothing depends on the machine it ran on.

*Why:* this converts the strongest available scepticism — "the tally was
manipulated" — into a check anyone can run on a laptop.

*What it defeats:* every challenge that depends on the inputs or the algorithm
being altered. If the observer's re-run matches the published hashes, the
manipulation claim has nothing left to attach to.

### 3. Public tooling, open source

The verifier, the transparency log, and the reproducibility bundle are
published for third parties to audit and to run. This repository is the
substrate for that.

*Why:* an assurance nobody can inspect is an assurance nobody should rely on.
Publishing the verification path is what makes the other decisions credible.

### 4. The record is the product

Every meaningful action — a voter check, a result capture, an override, an
adjudication, a configuration change — emits an append-only event carrying who
acted, where, when, on what device, and the before/after state hashes.

*Why:* the 2017 "declined access" finding arose from having no pre-agreed
protocol for producing the record under challenge. Making access a default and
the trail complete inverts that: the question becomes what is excluded from
the log, and the answer is direct biometric data only, by design and by law.

*What it defeats:* the hidden-server narrative, and any procedural challenge to
an individual decision.

### 5. Assumption audit over process

After results are declared, a verifiable random function seeded from the final
Merkle root selects the sample of stations subject to mandatory recount.

*Why:* when an operator chooses the audited sample, the audit is an argument
rather than a test. Deriving the sample from the data itself means any party
can compute the same selection independently, and nobody can steer it.

*What it defeats:* the bias challenge against the audit mechanism itself.

### 6. Failure must be pre-exercised

Election-day fallbacks — the manual results path, the chain-of-custody
protocol for physical ballots, the equipment reconciliation cycle — are
rehearsed in a full dry run beforehand, with pre-signed readiness
documentation at every site.

*Why:* a fallback that has never been exercised is not a fallback. The 2013
transmission failure was survivable; what made it damaging was discovering it
without a rehearsed alternative already in motion.

*What it defeats:* the "the technology failed and they improvised" narrative,
which recurs every cycle.

### 7. Sovereignty and access are design constraints

Data residency, jurisdiction, and supply-chain trust are architecture
requirements, not deployment preferences. Administrative access is
dual-controlled and logged. High-availability targets are tiered by what is
election-critical, cycle-critical, and routine.

*Why:* the 2022 nationality-and-access ground is a supply-chain argument.
Answering it structurally — least privilege, logged access, attested devices —
is more durable than answering it rhetorically.

---

## What this repository is

The implementation of the above: services, contracts, event schemas,
transparency and verification tooling, and the tests that hold them to it.

Where the reasoning runs deeper than a paragraph, it is recorded as an
architecture decision record under `docs/adr/`. Where a decision was
contested, the ADR records the alternatives that were rejected and why —
including the negative results, which are results.

## The honest limits

An audit trail proves what the system recorded. It cannot prove the system
recorded everything that happened, and no cryptographic scheme substitutes for
the physical chain of custody of ballot paper. Transparency reduces the space
in which a claim can be made credibly; it does not make every claim false.

This system is designed to make the questions that follow an election
answerable. It does not claim to make them unaskable.
