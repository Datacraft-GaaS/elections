# ADR 0001: Monorepo with per-module vertical slices

Status: accepted (initial scaffold)
Date: 2026-08-21

## Context

30 feature modules must ship on desktop, mobile, and server. Options: polyrepo per module, polyrepo per shell, monorepo grouped by shell, monorepo grouped by module.

## Decision

Single monorepo. Modules are the primary axis: `modules/<name>/{shared,server,desktop,mobile,docs}`. Shells (`apps/*`) are thin composition layers. Shared libraries live in `packages/*`.

## Consequences

+ One team can own a module end to end without coordinating across repos.
+ Contracts (`shared/`) are colocated with implementations, drift is visible in a single diff.
+ Refactors that touch all three platforms of a module are one PR.
- Repo-wide tooling (build graph, CI selectivity) becomes load-bearing early. Revisit if module count or team count grows past what a single graph can schedule cleanly.
- Requires strict discipline: no `mod-A` importing `mod-B`. Enforce with lint rules once the language is picked (ADR-0002).
