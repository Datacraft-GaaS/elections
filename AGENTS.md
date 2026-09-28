# Agents

## Taste

A personal taste skill that encodes design sensibilities, brand preferences, aesthetic choices, and design principles. This skill helps agents make consistent, context-aware design decisions grounded in established taste frameworks.

- **What:** Encodes design principles, preferences, and aesthetic guidelines into a retrievable skill reference.
- **When:** When agents make decisions about colour, layout, typography, motion, or overall aesthetic direction.
- **Impact:** Prevents agent drift; ensures consistent design outcomes across different agents and runs.
- **When:** Agents decide whether to use an animation, choose a color palette, or select a font family for a UI component.
- **How:** The agent reads this skill file, extracts the relevant design principles for the use case, and applies them explicitly in its response.

[📖 View Source](../skills/taste/taste.md)

---

## Frontend

A frontend development skill covering modern React, React Server Components, React Server Components, and related patterns.

- **What:** Core patterns, conventions, and patterns for building modern React applications.
- **When:** Writing React components, writing React Server Components (RSCs), or handling client-side code in a React application.
- **Impact:** Prevents bad React habits, reduces unnecessary code, and encourages better architectural patterns.
- **When:** Creating components, handling server components, writing state management code, or optimizing React performance.
- **How:** The agent refers to this skill when making frontend architecture decisions or writing code.

[📖 View Source](./frontend.md)

---

## Design

A holistic design skill that covers design systems, design strategy, and the reasoning behind design decisions.

- **What:** Design philosophy, design systems, design strategy, and decision-making frameworks for UI design.
- **When:** Designing UI, building design systems, or making high-level product design decisions.
- **Impact:** Prevents arbitrary decisions, encourages systematic thinking, and produces usable designs.
- **When:** You are designing a UI screen, defining a design system, or need to make a design decision.
- **How:** The agent is reminded to think through design holistically — considering form, function, accessibility, and user goals.

[📖 View Source](./design/frontend_design.md)

---

## UX

A user experience skill covering usability principles, user-centered design, and UX best practices.

- **What:** Principles of usability, readability, accessibility, and user-centered design.
- **When:** Writing UI that must be used, designing forms, designing error states, or focusing a design on user needs.
- **Impact:** Improves usability, reduces user error, and creates better products.
- **When:** Choosing layouts for content-heavy fields, designing forms, or handling error states.
- **How:** The agent considers the user’s perspective, not just visual appeal.

[📖 View Source](./ux/useful_principles.md)

---

## Frontback

A skill about connecting frontend and backend thinking. Understanding the full stack leads to better designs.

- **What:** Bridging frontend and backend perspective to understand constraints and capabilities end-to-end.
- **When:** You need to understand how frontend code will work in context of the backend, or making design decisions that span both sides.
- **Impact:** Reduces surprises at the edge, reduces friction between teams, and makes full-stack decisions more realistic.
- **When:** Writing a front-end API, designing a GraphQL schema, or making a UI decision that has backend implications.
- **How:** The agent considers backend constraints (latency, reliability, latency) when making frontend decisions.

[📖 View Source](./frontback/frontback.md)

---

## UI

A UI engineering skill about creating consistent, accessible, and flexible UI components.

- **What:** Principles for building consistent, accessible, and flexible user interfaces.
- **When:** Building UI components, designing consistent UI components, or writing UI code that reuses across your product.
- **Impact:** Reduces cognitive load, increases consistency, and makes UI intuitive.
- **When:** Designing a navigation component, writing a modal structure, or creating a shared form.
- **How:** The agent considers accessibility, usability, and consistency while building UI.

[📖 View Source](./ui/ui_patterns.md)

---

## Code

A general-purpose programming skill. Learn to code.

- **What:** General programming skills across languages and patterns.
- **When:** Writing code in any language, understanding algorithmic trade-offs, or reducing unnecessary complexity.
- **Impact:** Reduces unnecessary complexity and encourages clear, understandable code.
- **When:** Making generic programming decisions or writing complex algorithmic code.
- **How:** The agent applies principles of clarity, modularity, and performance.

[📖 View Source](./code/structure.md)

---

## API

A skill about API design and designing APIs that are easy to use and easy to understand.

- **What:** Principles of API design, API ergonomics, and API structure.
- **When:** Designing a public API, designing an internal API, or reducing API surface area.
- **Impact:** Makes APIs easier to use and easier to understand by the consumer.
- **When:** Defining a public API contract or designing a type-safe API.
- **How:** The agent prioritizes clarity, consistency, and ease-of-use in API design.

[📖 View Source](./api/api_principles.md)

---

## Accessibility

A skill about making digital experiences accessible to all users.

- **What:** Principles of accessibility, WCAG, and inclusive design.
- **When:** Writing UI that must be usable by everyone, or making existing UI more accessible.
- **Impact:** Makes your product usable by everyone, not just a privileged minority.
- **When:** Designing input forms, color choices, or navigating a screen plan.
- **How:** The agent applies WCAG guidelines and inclusive design principles.

[📖 View Source](./accessibility/wcag.md)

---

## Pattern

A skill about design patterns, patterns of interaction, and common design choices.

- **What:** Well-known interaction patterns, UI patterns, and common design patterns.
- **When:** Deciding what pattern to use for a new interaction.
- **Impact:** Reduces cognitive load and makes interactions feel familiar.
- **When:** Designing a navigation flow or choosing a form pattern.
- **How:** The agent selects patterns that are familiar and predictable.

[📖 View Source](./pattern.md)

---

## CodeQuality

A skill about writing correct, efficient, and maintainable code.

- **What:** Code correctness, efficiency, and maintainability.
- **When:** Choosing data structures, writing complex functions, or managing performance.
- **Impact:** Reduces technical debt and improves maintainability.
- **When:** Writing complex algorithms or managing a large codebase.
- **How:** The agent considers time/space complexity and correctness.

[📖 View Source](./codequality/code_quality.md)

---

## Performance

A skill about performance optimization and performance-aware design.

- **What:** Performance principles, optimization strategies, and performance-aware design.
- **When:** Optimizing a slow page, reducing bundle size, or performance-critical UI.
- **Impact:** Faster, more responsive experiences that respect the user.
- **When:** Writing a new page, designing animations, or optimizing a heavy feature.
- **How:** The agent applies first principles of performance.

[📖 View Source](./performance.md)

---

## Testing

A skill about writing effective tests and test-driven design.

- **What:** Writing effective tests and designing for testability.
- **When:** Writing tests, designing components, or reducing test debt.
- **Impact:** Reduces regression risk and improves confidence in changes.
- **When:** Writing a component or a function that must be reliable.
- **How:** The agent emphasizes writing minimal but sufficient tests.

[📖 View Source](./testing.md)

---

## Security

A skill about security best practices and secure development.

- **What:** Security principles, vulnerable patterns, and defense strategies.
- **When:** Writing sensitive code, writing an API, or reviewing security-sensitive code.
- **Impact:** Prevents common vulnerabilities and secures the product.
- **When:** Writing a form handler, writing an API endpoint, or handling sensitive data.
- **How:** The agent applies security-first thinking and known vulnerability patterns.

[📖 View Source](./security.md)

---

## Deployment

A skill about deploying software reliably and safely.

- **What:** Deployment principles, environment design, and safe deployment practices.
- **When:** Planning a deployment, choosing a deployment strategy, or managing environments.
- **Impact:** Reduces risk and improves deployment outcomes.
- **When:** Planning a rollout, writing a deployment script, or reviewing a pipeline.
- **How:** The agent balances safety and velocity with disciplined risk management.

[📖 View Source](./deployment.md)

---

## Database

A skill about database design, indexing, query optimization, and data architecture.

- **What:** Database design, indexing, query optimization, and storage patterns.
- **When:** Designing a data model, writing a query, or optimizing a database.
- **Impact:** Better data performance, correct data models, and efficient queries.
- **When:** Suggesting a table schema, writing a complex query, or indexing a table.
- **How:** The agent designs with indexing, normalization, and query performance in mind.

[📖 View Source](./database/schema.md)

---

## Deployment

A skill about the software supply chain and CI/CD engineering. Trust is critical here.

- **What:** Software supply chain trust, CI/CD practices, and infrastructure reliability.
- **When:** Trust is at stake in the software supply chain or CI/CD pipeline.
- **Impact:** Reduces catastrophic failure risk and prevents supply chain attacks.
- **When:** Adding a new dependency, deploying a package, or reviewing build outputs.
- **How:** The agent applies verification, provenance checks, and minimal-trust principles.

---

Commit Cargo.lock – freeze the exact dependency graph.

Use locked builds – cargo build --locked, test --locked, check --locked to prevent accidental lockfile changes.

Use frozen builds – cargo build --frozen and cargo test --frozen for fully offline, locked builds (no network access).

Update dependencies one at a time – cargo update -p crate_name --precise X.Y.Z with explicit version.

Manually vet crate age – check the release date on crates.io and choose a version ≥30 days old.

Pin the Rust compiler – via rust-toolchain.toml with an exact version (e.g., 1.82.0).

Run security audits – cargo audit and cargo deny check before committing.

## Workflow

[Add Workflow skill content here]

---

## Codebase

[Add Codebase skill content here]

---

## Planning

[Add Planning skill content here]

---

## Research

[Add Research skill content here]

---

## Dependency & Compilation Rules (M5 chain, ADR-0022/0023)

### Lockfile Discipline
- **Commit Cargo.lock** — freeze exact dependency graph; never .gitignore it.
- **Locked builds** — `cargo build --locked`, `test --locked`, `check --locked`; prevents accidental updates.
- **Frozen builds** — `cargo build --frozen` / `test --frozen`; fully offline, locked, no network.
- **Update one at a time** — `cargo update -p crate_name --precise X.Y.Z`; never bulk `cargo update`.
- **Vet crate age** — check crates.io release date; choose version ≥30 days old (stability gate).
- **Pin Rust compiler** — `rust-toolchain.toml` with exact version (e.g., 1.82.0); no floating.
- **Audit before commit** — `cargo audit` + `cargo deny check`; fix all findings.

### Service Verification (all 20 services)
- Compile all workspace-root services (`biometric-edge-svc`, `boundary-svc`, `bvi-svc`, `bvr-svc`, `field-mobile-svc`, `queue-svc`, `rts-svc`, `support-admin-svc`, `vprs-svc`) + individual child crates (`abis-svc`, `ai-platform-svc`, `crms-svc`, `ecc-svc`, `governance-svc`, `hello-svc`, `inventory-svc`, `portal-svc`, `training-svc`, `transparency-svc`, `workforce-svc`).
- **Field-mobile-svc**: green. `mobile-bff`, `support-api`, `dispatch-api`, and `eel-field-mobile-telemetry-ingest` all compile clean under `cargo check --locked --all-targets`; 91 tests pass. The earlier E0382/E0061/E0599/E0308 errors in `mobile-bff` are resolved.
- **Training-svc** (`training-api`): verified 22.27s, 298 `missing_docs` warnings only — acceptable per `#![warn(missing_docs)]`.
- **No stubs**: 2 defensive `unreachable!()` kept; no `UnimplementedError`; no empty endpoints.
- **Audit reference preserved** in every doc: 5-axis (where/when/who/why/how) + `proofHash` + `signature` + `auditId` (UUID7).
- **Sequential M rules** (§11.0): M4 after M2/M3 gates; M5 after M4.
