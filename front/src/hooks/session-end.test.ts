import test from 'node:test'
import assert from 'node:assert/strict'

import { sessionRealmAfterSessionEnd } from './session-end.ts'

test('an explicit logout clears the realm that issued the session', () => {
  assert.equal(sessionRealmAfterSessionEnd('acme', 'logout'), null)
})

test('an expired session keeps its realm for the next sign-in', () => {
  assert.equal(sessionRealmAfterSessionEnd('acme', 'expired'), 'acme')
})

test('a missing session realm stays missing on expiry', () => {
  assert.equal(sessionRealmAfterSessionEnd(null, 'expired'), null)
})
