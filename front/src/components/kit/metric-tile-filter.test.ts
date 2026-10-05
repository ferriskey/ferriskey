import test from 'node:test'
import assert from 'node:assert/strict'

import {
  isTileSelected,
  tileFilterKeys,
  tileFilterPatch,
  type TileFilter,
} from './metric-tile-filter.ts'

const tiles: { key: string; filter?: TileFilter }[] = [
  { key: 'total', filter: {} },
  { key: 'enabled', filter: { enabled: 'true' } },
  { key: 'disabled', filter: { enabled: 'false' } },
  { key: 'verified', filter: { email_verified: 'true' } },
  { key: 'pending' },
]

test('tileFilterKeys collects every key owned by the band once', () => {
  assert.deepEqual(tileFilterKeys(tiles), ['enabled', 'email_verified'])
})

test('a tile click clears the keys owned by the other tiles so filters never stack', () => {
  assert.deepEqual(tileFilterPatch(tiles, { enabled: 'false' }), {
    enabled: 'false',
    email_verified: '',
  })
  assert.deepEqual(tileFilterPatch(tiles, { email_verified: 'true' }), {
    enabled: '',
    email_verified: 'true',
  })
})

test('the all tile clears every key owned by the band', () => {
  assert.deepEqual(tileFilterPatch(tiles, {}), { enabled: '', email_verified: '' })
})

test('the patch leaves keys not owned by the band untouched', () => {
  assert.equal('search' in tileFilterPatch(tiles, { enabled: 'true' }), false)
})

test('a tile is selected when its patch matches the current filters', () => {
  const filters = { enabled: 'false', search: 'ada' }
  assert.equal(isTileSelected(tiles, { enabled: 'false' }, filters), true)
  assert.equal(isTileSelected(tiles, { enabled: 'true' }, filters), false)
  assert.equal(isTileSelected(tiles, {}, filters), false)
})

test('a tile is not selected when another band key is also set', () => {
  const filters = { enabled: 'false', email_verified: 'true' }
  assert.equal(isTileSelected(tiles, { enabled: 'false' }, filters), false)
  assert.equal(isTileSelected(tiles, { email_verified: 'true' }, filters), false)
})

test('the all tile is selected when no band key is set', () => {
  assert.equal(isTileSelected(tiles, {}, {}), true)
  assert.equal(isTileSelected(tiles, {}, { enabled: '', search: 'ada' }), true)
  assert.equal(isTileSelected(tiles, {}, { email_verified: 'true' }), false)
})
