import test from 'node:test'
import assert from 'node:assert/strict'

import {
  buildLoginErrorRedirect,
  validateCallbackParams,
} from './callback-helpers'

test('validateCallbackParams rejects missing authorization code', () => {
  assert.equal(
    validateCallbackParams({
      code: null,
      returnedState: 'state-1',
      expectedState: 'state-1',
    }),
    'Missing authorization code. Please try again.'
  )
})

test('validateCallbackParams rejects mismatched oauth state', () => {
  assert.equal(
    validateCallbackParams({
      code: 'code-1',
      returnedState: 'state-1',
      expectedState: 'state-2',
    }),
    'Invalid login state. Please try signing in again.'
  )
})

test('validateCallbackParams accepts a matching code and state', () => {
  assert.equal(
    validateCallbackParams({
      code: 'code-1',
      returnedState: 'state-1',
      expectedState: 'state-1',
    }),
    null
  )
})

test('buildLoginErrorRedirect encodes realm and login error', () => {
  assert.equal(
    buildLoginErrorRedirect('master', 'Token not found'),
    '/realms/master/authentication/login?login_error=Token+not+found'
  )
})
