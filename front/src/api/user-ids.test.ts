import test from 'node:test'
import assert from 'node:assert/strict'

import { USER_IDS_BATCH, idBatches } from './user-ids.ts'

const id = (n: number) => `00000000-0000-0000-0000-${String(n).padStart(12, '0')}`

test('no ids means no batch', () => {
  assert.deepEqual(idBatches([]), [])
})

test('duplicates collapse and order is stable', () => {
  assert.deepEqual(idBatches([id(2), id(1), id(2)]), [`${id(1)},${id(2)}`])
})

test('ids split into batches of at most the page limit', () => {
  const ids = Array.from({ length: 2 * USER_IDS_BATCH + 1 }, (_, n) => id(n))
  const batches = idBatches(ids)
  assert.equal(batches.length, 3)
  assert.deepEqual(
    batches.map((batch) => batch.split(',').length),
    [USER_IDS_BATCH, USER_IDS_BATCH, 1]
  )
  assert.equal(USER_IDS_BATCH, 100)
})
