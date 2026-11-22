import test from 'node:test'
import assert from 'node:assert/strict'

import { buildDocumentTitle } from './document-title.ts'

test('the title joins the page title and the realm display name', () => {
  assert.equal(buildDocumentTitle('Connexion', { displayName: 'Mon Entreprise', name: 'acme' }), 'Connexion – Mon Entreprise')
})

test('the realm name stands in when no display name is set', () => {
  assert.equal(buildDocumentTitle('Connexion', { displayName: null, name: 'acme' }), 'Connexion – acme')
  assert.equal(buildDocumentTitle('Connexion', { displayName: '   ', name: 'acme' }), 'Connexion – acme')
})

test('the title is the page title alone when the realm is unknown', () => {
  assert.equal(buildDocumentTitle('Connexion', null), 'Connexion')
})
