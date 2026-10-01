import assert from 'node:assert/strict'
import { describe, it } from 'node:test'
import {
  buildProviderUpdateBody,
  countProviderChanges,
  storedUsePkce,
  type ProviderDraft,
} from './provider-update-body.ts'

const stored: ProviderDraft = { displayName: 'Keycloak', enabled: true, usePkce: false }

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

describe('buildProviderUpdateBody', () => {
  it('omits use_pkce while the toggle has not moved', () => {
    const body = buildProviderUpdateBody({ ...stored, displayName: 'Renamed' }, stored)

    assert.deepEqual(body, { display_name: 'Renamed', enabled: true })
  })

  it('sends use_pkce once the toggle moved', () => {
    assert.equal(buildProviderUpdateBody({ ...stored, usePkce: true }, stored).use_pkce, true)
    assert.equal(
      buildProviderUpdateBody({ ...stored, usePkce: false }, { ...stored, usePkce: true })
        .use_pkce,
      false
    )
  })

  it('never sends the config back, so the masked secret cannot be stored', () => {
    const body = buildProviderUpdateBody({ ...stored, usePkce: true }, stored)

    assert.equal('config' in body, false)
  })
})

describe('countProviderChanges', () => {
  it('counts the pkce toggle alongside the other fields', () => {
    assert.equal(countProviderChanges(stored, stored), 0)
    assert.equal(countProviderChanges({ ...stored, usePkce: true }, stored), 1)
    assert.equal(
      countProviderChanges({ displayName: 'Other', enabled: false, usePkce: true }, stored),
      3
    )
  })
})
