import test from 'node:test'
import assert from 'node:assert/strict'

import {
  DEFAULT_LIMIT,
  nextSort,
  readListingState,
  toApiQuery,
  writeListingState,
} from './listing-query-state.ts'

const keys = ['username', 'enabled'] as const

test('an empty url gives the defaults', () => {
  assert.deepEqual(readListingState(new URLSearchParams(''), keys), {
    page: 1,
    limit: DEFAULT_LIMIT,
    sort: null,
    filters: {},
  })
})

test('a full url is read back', () => {
  const params = new URLSearchParams('page=3&limit=50&order_by=username&order=asc&username=jo&enabled=true&other=x')
  assert.deepEqual(readListingState(params, keys), {
    page: 3,
    limit: 50,
    sort: { orderBy: 'username', order: 'asc' },
    filters: { username: 'jo', enabled: 'true' },
  })
})

test('garbage page and limit fall back to the defaults', () => {
  const params = new URLSearchParams('page=0&limit=abc&order=up&order_by=name')
  const state = readListingState(params, keys)
  assert.equal(state.page, 1)
  assert.equal(state.limit, DEFAULT_LIMIT)
  assert.deepEqual(state.sort, { orderBy: 'name', order: 'desc' })
})

test('changing a filter resets the page and keeps unrelated params', () => {
  const params = new URLSearchParams('page=4&tab=x&username=jo')
  const next = writeListingState(params, { filters: { username: 'ja' } }, keys)
  assert.equal(next.get('page'), null)
  assert.equal(next.get('tab'), 'x')
  assert.equal(next.get('username'), 'ja')
})

test('clearing a filter removes it from the url', () => {
  const next = writeListingState(new URLSearchParams('username=jo'), { filters: { username: '' } }, keys)
  assert.equal(next.has('username'), false)
})

test('changing the sort resets the page', () => {
  const next = writeListingState(
    new URLSearchParams('page=4'),
    { sort: { orderBy: 'username', order: 'asc' } },
    keys,
  )
  assert.equal(next.get('page'), null)
  assert.equal(next.get('order_by'), 'username')
  assert.equal(next.get('order'), 'asc')
})

test('a null sort removes order_by and order', () => {
  const next = writeListingState(new URLSearchParams('order_by=username&order=asc'), { sort: null }, keys)
  assert.equal(next.has('order_by'), false)
  assert.equal(next.has('order'), false)
})

test('changing only the page keeps filters and sort', () => {
  const next = writeListingState(new URLSearchParams('username=jo&order_by=username&order=asc'), { page: 2 }, keys)
  assert.equal(next.get('page'), '2')
  assert.equal(next.get('username'), 'jo')
  assert.equal(next.get('order_by'), 'username')
})

test('page 1 and the default limit are not written', () => {
  const next = writeListingState(new URLSearchParams('page=2&limit=50'), { page: 1, limit: DEFAULT_LIMIT }, keys)
  assert.equal(next.has('page'), false)
  assert.equal(next.has('limit'), false)
})

test('sorting cycles asc, desc, default', () => {
  const asc = nextSort(null, 'username')
  assert.deepEqual(asc, { orderBy: 'username', order: 'asc' })
  const desc = nextSort(asc, 'username')
  assert.deepEqual(desc, { orderBy: 'username', order: 'desc' })
  assert.equal(nextSort(desc, 'username'), null)
  assert.deepEqual(nextSort(desc, 'email'), { orderBy: 'email', order: 'asc' })
})

test('the api query carries only what is set', () => {
  assert.deepEqual(toApiQuery({ page: 1, limit: 20, sort: null, filters: { username: 'jo', enabled: '' } }), {
    page: 1,
    limit: 20,
    username: 'jo',
  })
  assert.deepEqual(
    toApiQuery({ page: 2, limit: 50, sort: { orderBy: 'email', order: 'asc' }, filters: {} }),
    { page: 2, limit: 50, order_by: 'email', order: 'asc' },
  )
})
