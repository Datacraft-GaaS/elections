# ADR 0002: App stacks (desktop, mobile, server)

Status: proposed — needs input
Date: 2026-08-21

## Context

Need concrete tech choices for the three shells. Constraints not yet stated by the product owner.

## Options under consideration

- Desktop: Tauri (Rust core + web UI), Electron, native (Swift/Kotlin), Flutter desktop.
- Mobile: React Native, Flutter, native (Swift + Kotlin), Kotlin Multiplatform.
- Server: FastAPI (Python, per house style in `pjs/CLAUDE.md`), Go, Node/TS.

## Decision

Deferred. Pick before the first module implementation lands.

## Consequences

Blocks: `packages/sdk` shape, CI setup, lint/format toolchain.
