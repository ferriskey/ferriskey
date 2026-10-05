import test from 'node:test'
import assert from 'node:assert/strict'

import {
  boundsFromCalendarRange,
  calendarRangeFromBounds,
  fromRangeBounds,
  toRangeBounds,
  withRangeDay,
} from './date-range-bounds.ts'

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

const localDay = (year: number, month: number, date: number) => new Date(year, month - 1, date)

test('the calendar shows the stored bounds as the selected days', () => {
  assert.deepEqual(calendarRangeFromBounds('2026-10-05T00:00:00Z', '2026-10-08T00:00:00Z'), {
    from: localDay(2026, 10, 5),
    to: localDay(2026, 10, 7),
  })
  assert.deepEqual(calendarRangeFromBounds('2026-10-05T00:00:00Z', ''), {
    from: localDay(2026, 10, 5),
    to: undefined,
  })
  assert.equal(calendarRangeFromBounds('', ''), undefined)
  assert.equal(calendarRangeFromBounds('garbage', 'nope'), undefined)
})

test('a picked range sets an inclusive start and an exclusive end at UTC midnight', () => {
  assert.deepEqual(
    boundsFromCalendarRange({ from: '', to: '' }, { from: localDay(2026, 10, 5), to: localDay(2026, 10, 7) }),
    { from: '2026-10-05T00:00:00Z', to: '2026-10-08T00:00:00Z' },
  )
})

test('a range with only a start sets only the from bound', () => {
  assert.deepEqual(
    boundsFromCalendarRange({ from: '', to: '' }, { from: localDay(2026, 10, 5), to: undefined }),
    { from: '2026-10-05T00:00:00Z', to: '' },
  )
})

test('an unselected calendar clears both bounds', () => {
  assert.deepEqual(
    boundsFromCalendarRange({ from: '2026-10-05T00:00:00Z', to: '2026-10-08T00:00:00Z' }, undefined),
    { from: '', to: '' },
  )
})

test('a calendar edit keeps the raw stored bound of the untouched side', () => {
  assert.deepEqual(
    boundsFromCalendarRange(
      { from: '2026-10-01T06:00:00Z', to: '2026-10-08T10:30:00+02:00' },
      { from: localDay(2026, 10, 1), to: localDay(2026, 10, 12) },
    ),
    { from: '2026-10-01T06:00:00Z', to: '2026-10-13T00:00:00Z' },
  )
  assert.deepEqual(
    boundsFromCalendarRange(
      { from: '2026-10-01T06:00:00Z', to: '2026-10-08T10:30:00+02:00' },
      { from: localDay(2026, 9, 28), to: localDay(2026, 10, 8) },
    ),
    { from: '2026-09-28T00:00:00Z', to: '2026-10-08T10:30:00+02:00' },
  )
})
