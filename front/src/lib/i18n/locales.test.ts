import test from 'node:test'
import assert from 'node:assert/strict'

import { isSupportedLocale, normalizeLocale, pickSupportedLocale } from './locales.ts'

test('french is a supported locale', () => {
  assert.equal(isSupportedLocale('fr'), true)
})

test('regional french tags resolve to the fr catalogue', () => {
  assert.equal(normalizeLocale('fr'), 'fr')
  assert.equal(normalizeLocale('fr-FR'), 'fr')
  assert.equal(normalizeLocale('fr_CA'), 'fr')
  assert.equal(normalizeLocale('FR-be'), 'fr')
})

test('an accept-language list picks french ahead of an unsupported language', () => {
  assert.equal(pickSupportedLocale('de-DE,fr-CA;q=0.9,en;q=0.8'), 'fr')
})
