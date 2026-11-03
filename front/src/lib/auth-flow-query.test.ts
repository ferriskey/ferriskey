import test from 'node:test'
import assert from 'node:assert/strict'

import { authFlowSearch, hasAuthFlowClient, withAuthFlow } from './auth-flow-query.ts'

const fromApp = '?client_id=app-web&redirect_uri=https%3A%2F%2Fapp.example.com%2Fcb&state=s-1'

test('the client, redirect_uri and state of the sign-in are kept', () => {
  assert.equal(authFlowSearch(fromApp), fromApp)
})

test('anything else on the page is left behind', () => {
  assert.equal(
    authFlowSearch('?client_id=app-web&login_error=Wrong+password&execution=VERIFY_EMAIL'),
    '?client_id=app-web'
  )
})

test('a page reached without a sign-in carries nothing', () => {
  assert.equal(authFlowSearch(''), '')
  assert.equal(authFlowSearch('?token=abc'), '')
})

test('empty values are dropped rather than forwarded', () => {
  assert.equal(authFlowSearch('?client_id=app-web&state='), '?client_id=app-web')
})

test('a link keeps its path and gains the sign-in query', () => {
  assert.equal(withAuthFlow('../login', fromApp), `../login${fromApp}`)
  assert.equal(withAuthFlow('/realms/tenant/authentication/login', ''), '/realms/tenant/authentication/login')
})

test('a sign-in is for a client only when client_id is present', () => {
  assert.equal(hasAuthFlowClient(fromApp), true)
  assert.equal(hasAuthFlowClient('?state=s-1'), false)
  assert.equal(hasAuthFlowClient(''), false)
})
