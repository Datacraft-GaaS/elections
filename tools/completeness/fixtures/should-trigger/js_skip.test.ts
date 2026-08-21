// Fixture: should trigger js_skip

it.skip('not implemented', () => {
  expect(true).toBe(false);
});

describe.skip('whole suite pending', () => {
  it('also skipped', () => {});
});
