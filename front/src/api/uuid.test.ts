import test from 'node:test'
import assert from 'node:assert/strict'

import { isUuid } from './uuid.ts'

test('canonical uuids are recognised whatever their case', () => {
  assert.equal(isUuid('0199a4f2-3c1e-7b8a-9f00-1a2b3c4d5e6f'), true)
  assert.equal(isUuid('0199A4F2-3C1E-7B8A-9F00-1A2B3C4D5E6F'), true)
})

test('client ids and other strings are not uuids', () => {
  assert.equal(isUuid('admin-cli'), false)
  assert.equal(isUuid(''), false)
  assert.equal(isUuid('0199a4f2-3c1e-7b8a-9f00-1a2b3c4d5e6'), false)
})
