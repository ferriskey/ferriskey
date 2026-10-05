import test from 'node:test'
import assert from 'node:assert/strict'
import { keepPreviousData } from '@tanstack/react-query'

import { previousPagePlaceholder } from './paged-query.ts'

test('a paged listing keeps the previous page while the next one loads', () => {
  assert.equal(previousPagePlaceholder(true), keepPreviousData)
})

test('other queries keep the default placeholder behaviour', () => {
  assert.equal(previousPagePlaceholder(false), undefined)
  assert.equal(previousPagePlaceholder(), undefined)
})
