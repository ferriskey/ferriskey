import assert from 'node:assert/strict'
import { describe, it } from 'node:test'
import {
  buildProviderUpdateBody,
  countProviderChanges,
  extraConfigEntries,
  storedProviderDraft,
  storedUsePkce,
  validateProviderDraft,
  type ProviderDraft,
} from './provider-update-body.ts'

const provider = {
  display_name: 'Keycloak',
  enabled: true,
  config: {
    client_id: 'ferriskey',
    client_secret: '***',
    authorization_url: 'https://idp.example/auth',
    token_url: 'https://idp.example/token',
    userinfo_url: 'https://idp.example/userinfo',
    scopes: 'openid email profile',
    jwks_url: 'https://idp.example/jwks',
  },
}

const stored = storedProviderDraft(provider)

describe('storedProviderDraft', () => {
  it('reads the form fields out of the provider config', () => {
    assert.equal(stored.clientId, 'ferriskey')
    assert.equal(stored.authorizationUrl, 'https://idp.example/auth')
    assert.deepEqual(stored.scopes, ['openid', 'email', 'profile'])
    assert.equal(stored.usePkce, false)
  })

  it('never seeds the secret field with the mask the api returns', () => {
    assert.equal(stored.clientSecret, '')
  })

  it('survives a provider with no config at all', () => {
    const empty = storedProviderDraft({ display_name: null, enabled: false })

    assert.equal(empty.clientId, '')
    assert.deepEqual(empty.scopes, [])
    assert.equal(empty.displayName, '')
  })
})

describe('storedUsePkce', () => {
  it('reads the flag out of the provider config', () => {
    assert.equal(storedUsePkce({ use_pkce: true }), true)
    assert.equal(storedUsePkce({ use_pkce: false }), false)
  })

  it('treats an absent flag as disabled, like the broker does', () => {
    assert.equal(storedUsePkce({ client_id: 'ferriskey' }), false)
  })

  it('survives a config that is not an object', () => {
    assert.equal(storedUsePkce(null), false)
    assert.equal(storedUsePkce('nope'), false)
  })
})

describe('extraConfigEntries', () => {
  it('keeps only the keys the form does not own', () => {
    assert.deepEqual(extraConfigEntries(provider.config), [
      ['jwks_url', 'https://idp.example/jwks'],
    ])
  })
})

describe('validateProviderDraft', () => {
  it('accepts the stored provider as it stands', () => {
    assert.deepEqual(validateProviderDraft(stored), {})
  })

  it('refuses an emptied client id or endpoint', () => {
    const errors = validateProviderDraft({
      ...stored,
      clientId: '',
      authorizationUrl: '',
      tokenUrl: '',
    })

    assert.equal(errors.clientId, 'validation.client_id_required')
    assert.equal(errors.authorizationUrl, 'validation.url_required')
    assert.equal(errors.tokenUrl, 'validation.url_required')
  })

  it('refuses a malformed url but allows an emptied userinfo url', () => {
    assert.equal(
      validateProviderDraft({ ...stored, tokenUrl: 'idp.example/token' }).tokenUrl,
      'validation.url_invalid'
    )
    assert.equal(validateProviderDraft({ ...stored, userinfoUrl: '' }).userinfoUrl, undefined)
    assert.equal(
      validateProviderDraft({ ...stored, userinfoUrl: 'nope' }).userinfoUrl,
      'validation.url_invalid'
    )
  })
})

describe('buildProviderUpdateBody', () => {
  it('sends nothing but the two always-present fields when nothing moved', () => {
    assert.deepEqual(buildProviderUpdateBody(stored, stored), {
      display_name: 'Keycloak',
      enabled: true,
    })
  })

  it('sends the secret only when one was typed', () => {
    assert.equal(buildProviderUpdateBody(stored, stored).client_secret, undefined)
    assert.equal(
      buildProviderUpdateBody({ ...stored, clientSecret: 'rotated' }, stored).client_secret,
      'rotated'
    )
  })

  it('never sends the config object, so the masked secret cannot be stored', () => {
    const body = buildProviderUpdateBody({ ...stored, clientId: 'other' }, stored)

    assert.equal('config' in body, false)
    assert.equal(body.client_id, 'other')
  })

  it('sends an emptied userinfo url so the key can be dropped', () => {
    assert.equal(buildProviderUpdateBody({ ...stored, userinfoUrl: '' }, stored).userinfo_url, '')
  })

  it('sends scopes space joined, and only when the list moved', () => {
    assert.equal(buildProviderUpdateBody(stored, stored).scopes, undefined)
    assert.equal(
      buildProviderUpdateBody({ ...stored, scopes: ['openid', 'groups'] }, stored).scopes,
      'openid groups'
    )
  })
})

describe('countProviderChanges', () => {
  it('counts nothing on an untouched draft', () => {
    assert.equal(countProviderChanges(stored, stored), 0)
  })

  it('counts a typed secret even though the stored one is unknown', () => {
    assert.equal(countProviderChanges({ ...stored, clientSecret: 'rotated' }, stored), 1)
  })

  it('counts every moved field', () => {
    const draft: ProviderDraft = {
      displayName: 'Other',
      enabled: false,
      clientId: 'other',
      clientSecret: 'rotated',
      authorizationUrl: 'https://other.example/auth',
      tokenUrl: 'https://other.example/token',
      userinfoUrl: '',
      scopes: ['openid'],
      usePkce: true,
    }

    assert.equal(countProviderChanges(draft, stored), 9)
  })
})
