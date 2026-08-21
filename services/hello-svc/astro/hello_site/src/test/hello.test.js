import { describe, it, expect } from 'vitest'

describe('eel hello site', () => {
  it('greeting payload has ok: true', () => {
    const data = { ok: true }
    expect(data.ok).toBe(true)
  })
})
