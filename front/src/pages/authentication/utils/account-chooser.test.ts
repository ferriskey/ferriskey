import test from 'node:test'
import assert from 'node:assert/strict'

import {
  buildLoginPath,
  formatLastUsed,
  parseMaxAge,
  resolveSelectOutcome,
} from './account-chooser.ts'

test('max_age is read only when it is a positive integer', () => {
  assert.equal(parseMaxAge('?client_id=c&max_age=300'), 300)
  assert.equal(parseMaxAge('?max_age=0'), undefined)
  assert.equal(parseMaxAge('?max_age=-5'), undefined)
  assert.equal(parseMaxAge('?max_age=abc'), undefined)
  assert.equal(parseMaxAge('?client_id=c'), undefined)
})

test('the login path keeps the authorization query and drops max_age', () => {
  assert.equal(
    buildLoginPath('demo', '?client_id=c&redirect_uri=https%3A%2F%2Frp%2Fcb&state=s&max_age=60'),
    '/realms/demo/authentication/login?client_id=c&redirect_uri=https%3A%2F%2Frp%2Fcb&state=s'
  )
})

test('a login hint prefills the login path and replaces an incoming one', () => {
  assert.equal(
    buildLoginPath('demo', '?client_id=c&login_hint=old', 'a+b@x.io'),
    '/realms/demo/authentication/login?client_id=c&login_hint=a%2Bb%40x.io'
  )
})

test('the login path without a query has no trailing question mark', () => {
  assert.equal(buildLoginPath('demo', ''), '/realms/demo/authentication/login')
})

test('a resumed session follows its redirect url', () => {
  assert.deepEqual(resolveSelectOutcome({ status: 'Success', url: 'https://rp/cb?code=1' }), {
    kind: 'redirect',
    url: 'https://rp/cb?code=1',
  })
})

test('a refusal goes to the login page with the hint', () => {
  assert.deepEqual(
    resolveSelectOutcome({ status: 'requires_credentials', login_hint: 'alice' }),
    { kind: 'login', loginHint: 'alice' }
  )
})

test('a pending required action goes to the required action page', () => {
  assert.deepEqual(
    resolveSelectOutcome({ status: 'RequiresActions', required_actions: ['verify_email'] }),
    { kind: 'required-action', action: 'verify_email' }
  )
})

test('anything else is a failure', () => {
  assert.deepEqual(resolveSelectOutcome({ status: 'Failed' }), { kind: 'failed' })
  assert.deepEqual(resolveSelectOutcome({ status: 'Success', url: null }), { kind: 'failed' })
  assert.deepEqual(resolveSelectOutcome(undefined), { kind: 'failed' })
})

test('the last use is shown relative to now', () => {
  const now = Date.parse('2026-06-10T12:00:00Z')

  assert.equal(formatLastUsed('2026-06-10T11:59:40Z', now, 'en'), 'now')
  assert.equal(formatLastUsed('2026-06-10T11:55:00Z', now, 'en'), '5 minutes ago')
  assert.equal(formatLastUsed('2026-06-10T09:00:00Z', now, 'en'), '3 hours ago')
  assert.equal(formatLastUsed('2026-06-08T12:00:00Z', now, 'en'), '2 days ago')
})

test('an unreadable or absent date shows nothing', () => {
  assert.equal(formatLastUsed(undefined, 0, 'en'), null)
  assert.equal(formatLastUsed(null, 0, 'en'), null)
  assert.equal(formatLastUsed('not a date', 0, 'en'), null)
})
