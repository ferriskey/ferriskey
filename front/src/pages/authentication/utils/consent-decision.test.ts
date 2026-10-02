import assert from 'node:assert/strict'
import { describe, it } from 'node:test'
import {
  approvedScopesForAllow,
  approvedScopesForDeny,
  approvedScopesFromFormData,
  defaultApprovedOptionalScopes,
  toggleOptionalScope,
  type ConsentScopeView,
} from './consent-decision.ts'

const profile: ConsentScopeView = { name: 'profile', description: 'View your basic profile' }
const email: ConsentScopeView = { name: 'email', description: 'View your email address' }
const optionalScopes: ConsentScopeView[] = [profile, email]

describe('defaultApprovedOptionalScopes', () => {
  it('starts every optional scope approved, on by default', () => {
    assert.deepEqual(defaultApprovedOptionalScopes(optionalScopes), new Set(['profile', 'email']))
  })

  it('returns an empty set when there are no optional scopes', () => {
    assert.deepEqual(defaultApprovedOptionalScopes([]), new Set())
  })
})

describe('toggleOptionalScope', () => {
  it('removes a scope that was approved', () => {
    const approved = new Set(['profile', 'email'])
    assert.deepEqual(toggleOptionalScope(approved, 'profile'), new Set(['email']))
  })

  it('adds a scope that was not approved', () => {
    const approved = new Set(['email'])
    assert.deepEqual(toggleOptionalScope(approved, 'profile'), new Set(['profile', 'email']))
  })

  it('never mutates the set it was given', () => {
    const approved = new Set(['profile'])
    toggleOptionalScope(approved, 'profile')
    assert.deepEqual(approved, new Set(['profile']))
  })
})

describe('approvedScopesForAllow', () => {
  it('keeps only the optional scopes currently approved, in catalog order', () => {
    const approved = new Set(['email'])
    assert.deepEqual(approvedScopesForAllow(optionalScopes, approved), ['email'])
  })

  it('returns every optional scope when all are approved', () => {
    const approved = new Set(['profile', 'email'])
    assert.deepEqual(approvedScopesForAllow(optionalScopes, approved), ['profile', 'email'])
  })

  it('drops an approved name that is not in the optional scope list', () => {
    const approved = new Set(['profile', 'unknown'])
    assert.deepEqual(approvedScopesForAllow(optionalScopes, approved), ['profile'])
  })

  it('returns an empty array once every toggle has been switched off', () => {
    assert.deepEqual(approvedScopesForAllow(optionalScopes, new Set()), [])
  })
})

describe('approvedScopesForDeny', () => {
  it('is always empty, regardless of what was toggled', () => {
    assert.deepEqual(approvedScopesForDeny(), [])
  })
})

describe('approvedScopesFromFormData', () => {
  it('reads the checked approved_scopes entries off the form', () => {
    const data = new FormData()
    data.append('approved_scopes', 'email')
    assert.deepEqual(approvedScopesFromFormData(optionalScopes, data), ['email'])
  })

  it('supports multiple checked scopes, in catalog order', () => {
    const data = new FormData()
    data.append('approved_scopes', 'email')
    data.append('approved_scopes', 'profile')
    assert.deepEqual(approvedScopesFromFormData(optionalScopes, data), ['profile', 'email'])
  })

  it('returns an empty array when nothing is checked', () => {
    const data = new FormData()
    assert.deepEqual(approvedScopesFromFormData(optionalScopes, data), [])
  })

  it('ignores a checked value that is not a known optional scope', () => {
    const data = new FormData()
    data.append('approved_scopes', 'unknown')
    assert.deepEqual(approvedScopesFromFormData(optionalScopes, data), [])
  })
})
