import test from 'node:test'
import assert from 'node:assert/strict'

import { realmFromToken, sessionRealm } from './session-realm.ts'

function token(claims: Record<string, unknown>): string {
  const encode = (value: unknown) =>
    btoa(JSON.stringify(value)).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '')
  return `${encode({ alg: 'RS256', typ: 'JWT' })}.${encode(claims)}.signature`
}

test('the realm is the last segment of the issuer', () => {
  assert.equal(realmFromToken(token({ iss: 'https://auth.example.com/realms/master' })), 'master')
})

test('an issuer behind a path prefix still names its realm', () => {
  assert.equal(realmFromToken(token({ iss: 'https://example.com/auth/realms/clients' })), 'clients')
})

test('a trailing slash on the issuer is tolerated', () => {
  assert.equal(realmFromToken(token({ iss: 'http://localhost:3333/realms/master/' })), 'master')
})

test('a percent-encoded realm name is decoded', () => {
  assert.equal(
    realmFromToken(token({ iss: 'http://localhost:3333/realms/my%20realm' })),
    'my realm'
  )
})

test('a token without a realm issuer gives no realm', () => {
  assert.equal(realmFromToken(token({ iss: 'https://accounts.example.com' })), null)
  assert.equal(realmFromToken(token({ sub: 'no-issuer' })), null)
  assert.equal(realmFromToken(token({ iss: 42 })), null)
})

test('a missing or malformed token gives no realm', () => {
  assert.equal(realmFromToken(null), null)
  assert.equal(realmFromToken(undefined), null)
  assert.equal(realmFromToken(''), null)
  assert.equal(realmFromToken('not-a-jwt'), null)
  assert.equal(realmFromToken('header.%%%.signature'), null)
})

test('the session realm wins over the browsed realm', () => {
  const access = token({ iss: 'http://localhost:3333/realms/master' })
  assert.equal(sessionRealm(access, null, 'clients'), 'master')
})

test('the refresh token answers when the access token is gone', () => {
  const refresh = token({ iss: 'http://localhost:3333/realms/master' })
  assert.equal(sessionRealm(null, refresh, 'clients'), 'master')
})

test('without a session the browsed realm is used', () => {
  assert.equal(sessionRealm(null, null, 'clients'), 'clients')
})
