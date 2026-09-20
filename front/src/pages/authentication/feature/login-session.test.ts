import test from 'node:test'
import assert from 'node:assert/strict'

import { resolveLoginSession } from './login-session.ts'

test('stored tokens are discarded when the server sent us back with an authorization request', () => {
  assert.equal(
    resolveLoginSession({
      isAuthenticated: true,
      isAuthInitiated: true,
      sessionExpired: false,
    }),
    'discard-stale-session'
  )
})

test('stored tokens are discarded when the server marked the session as expired', () => {
  assert.equal(
    resolveLoginSession({
      isAuthenticated: true,
      isAuthInitiated: false,
      sessionExpired: true,
    }),
    'discard-stale-session'
  )
})

test('an expired-session marker is honoured even with nothing left to discard', () => {
  assert.equal(
    resolveLoginSession({
      isAuthenticated: false,
      isAuthInitiated: false,
      sessionExpired: true,
    }),
    'discard-stale-session'
  )
})

test('an authenticated visitor with no pending authorization request goes to the console', () => {
  assert.equal(
    resolveLoginSession({
      isAuthenticated: true,
      isAuthInitiated: false,
      sessionExpired: false,
    }),
    'enter-console'
  )
})

test('an anonymous visitor carrying an authorization request gets the login form', () => {
  assert.equal(
    resolveLoginSession({
      isAuthenticated: false,
      isAuthInitiated: true,
      sessionExpired: false,
    }),
    'show-login-form'
  )
})

test('an anonymous visitor with nothing in the query gets the login form', () => {
  assert.equal(
    resolveLoginSession({
      isAuthenticated: false,
      isAuthInitiated: false,
      sessionExpired: false,
    }),
    'show-login-form'
  )
})
