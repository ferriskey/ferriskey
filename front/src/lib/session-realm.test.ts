import test from 'node:test'
import assert from 'node:assert/strict'

import { realmFromIssuer } from './session-realm.ts'

test('the realm is the last segment of the issuer', () => {
  assert.equal(realmFromIssuer('https://auth.example.com/realms/master'), 'master')
})

test('an issuer behind a path prefix still names its realm', () => {
  assert.equal(realmFromIssuer('https://example.com/auth/realms/clients'), 'clients')
})

test('a trailing slash on the issuer is tolerated', () => {
  assert.equal(realmFromIssuer('http://localhost:3333/realms/master/'), 'master')
})

test('every character a realm name may hold is kept', () => {
  assert.equal(realmFromIssuer('http://localhost:3333/realms/Tenant_01-eu'), 'Tenant_01-eu')
})

test('an issuer that names no realm gives no realm', () => {
  assert.equal(realmFromIssuer('https://accounts.example.com'), null)
  assert.equal(realmFromIssuer('http://localhost:3333/realms/'), null)
  assert.equal(realmFromIssuer('http://localhost:3333/realms/master/extra'), null)
})

test('a missing issuer gives no realm', () => {
  assert.equal(realmFromIssuer(null), null)
  assert.equal(realmFromIssuer(undefined), null)
  assert.equal(realmFromIssuer(''), null)
})
