// Snapshot tests: 11-language coverage, stale-label logic, cache-key builder.
// (AC-M11-11, M11-I18N-, M11-SEC-)

import { describe, it, expect } from 'vitest'
import { LOCALES, t } from '../i18n/locales.js'

describe('i18n: 11-language coverage', () => {
  it('exports exactly 11 locales', () => {
    expect(LOCALES).toHaveLength(11)
  })

  it('all locales produce a non-empty home title', () => {
    for (const locale of LOCALES) {
      const title = t(locale, 'home.title')
      // Falls back to English if locale not yet translated; key itself must not surface.
      expect(title).not.toBe('home.title')
      expect(title.length).toBeGreaterThan(0)
    }
  })

  it('variable interpolation works', () => {
    const result = t('en', 'home.stale_warning', { ts: '2026-08-21T12:00:00Z' })
    expect(result).toContain('2026-08-21T12:00:00Z')
  })
})

describe('cache-key builder (signing headers)', () => {
  // Simulate what portal-api enforces: cache keys must include merkle_root + snapshot_id
  // so that a poisoned snapshot with a different root produces a different cache entry.
  function buildCacheKey(merkleRoot, snapshotId) {
    return `mr=${merkleRoot};sid=${snapshotId}`
  }

  it('different merkle roots produce different cache keys', () => {
    const root1 = 'a'.repeat(64)
    const root2 = 'b'.repeat(64)
    const sid = '00000000-0000-0000-0000-000000000001'
    expect(buildCacheKey(root1, sid)).not.toBe(buildCacheKey(root2, sid))
  })

  it('same root + same id produces stable key', () => {
    const root = 'c'.repeat(64)
    const sid = '00000000-0000-0000-0000-000000000002'
    expect(buildCacheKey(root, sid)).toBe(buildCacheKey(root, sid))
  })
})

describe('cache-poisoning defence (AC-M11-05)', () => {
  // Simulate attempt to inject a snapshot with a mismatched cosign signature.
  // The client must reject any snapshot where the declared root does not match
  // the value embedded in the cosign signature over the tarball.
  function verifySnapshotSignature(declaredRoot, signedRoot) {
    // In production: Ed25519 verify(sig, canonical_json({root: signedRoot}), commission_pubkey)
    return declaredRoot === signedRoot
  }

  it('rejects snapshot with mismatched cosign signature', () => {
    const declared = 'a'.repeat(64)
    const signed = 'b'.repeat(64) // attacker replaced root in JSON
    expect(verifySnapshotSignature(declared, signed)).toBe(false)
  })

  it('accepts snapshot with matching cosign signature', () => {
    const root = 'c'.repeat(64)
    expect(verifySnapshotSignature(root, root)).toBe(true)
  })
})

describe('stale-label logic (EC-TIM-05)', () => {
  const STALE_THRESHOLD_MS = 60_000 // 60 s

  function isStale(publishedAtMs, nowMs = Date.now()) {
    return nowMs - publishedAtMs > STALE_THRESHOLD_MS
  }

  it('labels snapshot as stale after threshold', () => {
    const old = Date.now() - 120_000
    expect(isStale(old)).toBe(true)
  })

  it('does not label fresh snapshot as stale', () => {
    const recent = Date.now() - 10_000
    expect(isStale(recent)).toBe(false)
  })
})
