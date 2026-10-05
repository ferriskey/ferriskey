import test from 'node:test'
import assert from 'node:assert/strict'

import {
  activeFieldCount,
  clearColumnFilters,
  columnFilterIndicator,
  countActiveFilters,
  fieldKeys,
  type ColumnFilterField,
} from './column-filter-state.ts'

const enabled: ColumnFilterField = { kind: 'boolean', key: 'enabled', label: 'Enabled' }
const username: ColumnFilterField = { kind: 'text', key: 'username', label: 'Username' }
const created: ColumnFilterField = {
  kind: 'date-range',
  fromKey: 'created_from',
  toKey: 'created_to',
  label: 'Created',
}
const status: ColumnFilterField = {
  kind: 'enum',
  key: 'status',
  label: 'Status',
  options: [{ value: 'open', label: 'Open' }],
}

test('a date range exposes both of its keys', () => {
  assert.deepEqual(fieldKeys(created), ['created_from', 'created_to'])
  assert.deepEqual(fieldKeys(username), ['username'])
})

test('no active filter gives no indicator', () => {
  assert.deepEqual(columnFilterIndicator([enabled, username], {}), { kind: 'none' })
  assert.deepEqual(columnFilterIndicator([enabled, username], { username: '', enabled: '' }), { kind: 'none' })
})

test('a single active boolean shows its value', () => {
  assert.deepEqual(columnFilterIndicator([enabled, username], { enabled: 'true' }), {
    kind: 'value',
    key: 'enabled',
    value: 'true',
  })
  assert.deepEqual(columnFilterIndicator([enabled], { enabled: 'false' }), {
    kind: 'value',
    key: 'enabled',
    value: 'false',
  })
})

test('a single active text filter shows the active icon', () => {
  assert.deepEqual(columnFilterIndicator([enabled, username], { username: 'jo' }), { kind: 'active' })
})

test('two active filters show a count', () => {
  assert.deepEqual(columnFilterIndicator([enabled, username], { enabled: 'true', username: 'jo' }), {
    kind: 'count',
    count: 2,
  })
})

test('a date range with only its lower bound is one active filter', () => {
  assert.equal(activeFieldCount([created], { created_from: '2026-10-05T00:00:00Z' }), 1)
  assert.deepEqual(columnFilterIndicator([created], { created_from: '2026-10-05T00:00:00Z' }), { kind: 'active' })
})

test('a date range with both bounds counts once', () => {
  assert.equal(
    activeFieldCount([created], { created_from: '2026-10-01T00:00:00Z', created_to: '2026-10-05T00:00:00Z' }),
    1,
  )
})

test('active filters are counted across every column regardless of visibility', () => {
  const columns = [{ filters: [enabled, username] }, {}, { filters: [created, status] }]
  assert.equal(
    countActiveFilters(columns, { enabled: 'true', username: '', created_to: '2026-10-05T00:00:00Z', status: 'open' }),
    3,
  )
})

test('clearing a column only resets that column keys', () => {
  assert.deepEqual(clearColumnFilters([enabled, created]), {
    enabled: '',
    created_from: '',
    created_to: '',
  })
})
