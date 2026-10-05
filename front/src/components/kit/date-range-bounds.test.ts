import test from 'node:test'
import assert from 'node:assert/strict'

import { fromRangeBounds, toRangeBounds, withRangeDay } from './date-range-bounds.ts'

test('from is the start of the picked day in UTC and to is the start of the next day', () => {
  assert.deepEqual(toRangeBounds('2026-10-05', '2026-10-07'), {
    from: '2026-10-05T00:00:00Z',
    to: '2026-10-08T00:00:00Z',
  })
})

test('a to day at the end of a month and a year rolls over', () => {
  assert.deepEqual(toRangeBounds('2026-02-28', '2026-12-31'), {
    from: '2026-02-28T00:00:00Z',
    to: '2027-01-01T00:00:00Z',
  })
  assert.equal(toRangeBounds('', '2028-02-28').to, '2028-02-29T00:00:00Z')
})

test('an empty or malformed day yields an empty bound', () => {
  assert.deepEqual(toRangeBounds('', ''), { from: '', to: '' })
  assert.deepEqual(toRangeBounds('2026-10', 'nope'), { from: '', to: '' })
  assert.deepEqual(toRangeBounds('2026-02-30', '2026-13-01'), { from: '', to: '' })
})

test('the inverse shows the picked days back', () => {
  assert.deepEqual(fromRangeBounds('2026-10-05T00:00:00Z', '2026-10-08T00:00:00Z'), {
    fromDay: '2026-10-05',
    toDay: '2026-10-07',
  })
  assert.deepEqual(fromRangeBounds('2027-01-01T00:00:00Z', '2026-03-01T00:00:00Z'), {
    fromDay: '2027-01-01',
    toDay: '2026-02-28',
  })
})

test('the inverse reads offsets in UTC and ignores empty or invalid bounds', () => {
  assert.deepEqual(fromRangeBounds('2026-10-05T01:00:00+02:00', ''), {
    fromDay: '2026-10-04',
    toDay: '',
  })
  assert.deepEqual(fromRangeBounds('garbage', ''), { fromDay: '', toDay: '' })
})

test('round trip keeps the picked days', () => {
  const bounds = toRangeBounds('2026-01-01', '2026-01-31')
  assert.deepEqual(fromRangeBounds(bounds.from, bounds.to), {
    fromDay: '2026-01-01',
    toDay: '2026-01-31',
  })
})

test('editing the from day keeps the raw stored to bound untouched', () => {
  assert.deepEqual(
    withRangeDay({ from: '', to: '2026-10-08T10:30:00+02:00' }, 'from', '2026-10-01'),
    { from: '2026-10-01T00:00:00Z', to: '2026-10-08T10:30:00+02:00' },
  )
})

test('editing the to day keeps the raw stored from bound untouched', () => {
  assert.deepEqual(
    withRangeDay({ from: '2026-10-01T06:00:00Z', to: '' }, 'to', '2026-10-07'),
    { from: '2026-10-01T06:00:00Z', to: '2026-10-08T00:00:00Z' },
  )
})

test('clearing one day empties only that bound', () => {
  assert.deepEqual(
    withRangeDay({ from: '2026-10-01T06:00:00Z', to: '2026-10-08T10:00:00Z' }, 'to', ''),
    { from: '2026-10-01T06:00:00Z', to: '' },
  )
})
