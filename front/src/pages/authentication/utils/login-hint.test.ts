import test from 'node:test'
import assert from 'node:assert/strict'

import { readLoginHint } from './login-hint.ts'

test('the login hint is read from the query', () => {
  assert.equal(readLoginHint('?client_id=c&login_hint=a%2Bb%40x.io'), 'a+b@x.io')
})

test('an absent or blank hint gives an empty username', () => {
  assert.equal(readLoginHint(''), '')
  assert.equal(readLoginHint('?client_id=c'), '')
  assert.equal(readLoginHint('?login_hint=%20%20'), '')
})
