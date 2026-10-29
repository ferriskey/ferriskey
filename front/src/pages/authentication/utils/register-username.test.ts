import assert from 'node:assert/strict'
import { describe, it } from 'node:test'
import { resolveRegisterUsername } from './register-username.ts'

describe('resolveRegisterUsername', () => {
  it('keeps the username the form provides', () => {
    assert.equal(resolveRegisterUsername('jdoe', 'contact@a.fr'), 'jdoe')
  })

  it('trims the username the form provides', () => {
    assert.equal(resolveRegisterUsername('  jdoe  ', 'contact@a.fr'), 'jdoe')
  })

  it('falls back to the full email when the form has no username', () => {
    assert.equal(resolveRegisterUsername('', 'contact@a.fr'), 'contact@a.fr')
  })

  it('falls back to the full email when the username is blank', () => {
    assert.equal(resolveRegisterUsername('   ', 'contact@a.fr'), 'contact@a.fr')
  })

  it('never reduces the email to its local part', () => {
    assert.notEqual(resolveRegisterUsername('', 'contact@a.fr'), 'contact')
    assert.notEqual(
      resolveRegisterUsername('', 'contact@b.fr'),
      resolveRegisterUsername('', 'contact@a.fr'),
    )
  })

  it('lowercases and trims the email the way the server stores it', () => {
    assert.equal(resolveRegisterUsername('', '  Contact@A.fr '), 'contact@a.fr')
  })
})
