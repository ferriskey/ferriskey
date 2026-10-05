import test from 'node:test'
import assert from 'node:assert/strict'

import { inferApplicationType, type ApplicationShape } from './infer-application-type.ts'

const REDIRECT = [{ value: 'https://app.test/callback' }]

const client = (patch: Partial<ApplicationShape>): ApplicationShape => ({
  service_account_enabled: false,
  oauth_device_code_grant_enabled: false,
  client_type: 'confidential',
  public_client: false,
  redirect_uris: [],
  ...patch,
})

test('a service account makes a machine-to-machine application', () => {
  assert.equal(
    inferApplicationType(client({ service_account_enabled: true, oauth_device_code_grant_enabled: true })),
    'm2m'
  )
})

test('the device grant without redirect uri makes a device application', () => {
  assert.equal(inferApplicationType(client({ oauth_device_code_grant_enabled: true })), 'device')
})

test('a device-grant client with a redirect uri is classed by its client type', () => {
  assert.equal(
    inferApplicationType(
      client({
        oauth_device_code_grant_enabled: true,
        client_type: 'public',
        public_client: true,
        redirect_uris: REDIRECT,
      })
    ),
    'spa'
  )
  assert.equal(
    inferApplicationType(client({ oauth_device_code_grant_enabled: true, redirect_uris: REDIRECT })),
    'web'
  )
})

test('public client types split between spa and native', () => {
  assert.equal(inferApplicationType(client({ client_type: 'public', public_client: true })), 'spa')
  assert.equal(inferApplicationType(client({ client_type: 'public', public_client: false })), 'native')
})

test('every other client type is a web application', () => {
  assert.equal(inferApplicationType(client({ client_type: 'system' })), 'web')
  assert.equal(inferApplicationType(client({})), 'web')
})
