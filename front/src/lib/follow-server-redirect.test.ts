import test from 'node:test'
import assert from 'node:assert/strict'

import { isNavigableRedirect } from './follow-server-redirect.ts'

test('web and relative targets are followed', () => {
  for (const url of [
    'https://app.example.com/callback?code=abc',
    'http://localhost:5555/realms/master/authentication/callback',
    '/realms/master/authentication/login?login_error=x',
    'callback?code=abc',
  ]) {
    assert.equal(isNavigableRedirect(url), true, url)
  }
})

test('native application schemes are followed', () => {
  for (const url of ['myapp://callback?code=abc', 'com.example.app:/oauth2redirect?code=abc']) {
    assert.equal(isNavigableRedirect(url), true, url)
  }
})

test('targets that would run script or load a document from the page are refused', () => {
  for (const url of [
    'javascript:alert(1)',
    '  JavaScript:alert(1)',
    'java\nscript:alert(1)',
    'java\tscript:alert(1)',
    'data:text/html,<script>alert(1)</script>',
    'vbscript:msgbox(1)',
    'blob:https://app.example.com/uuid',
    'file:///etc/passwd',
    'about:blank',
  ]) {
    assert.equal(isNavigableRedirect(url), false, JSON.stringify(url))
  }
})

test('an empty or non string target is refused', () => {
  assert.equal(isNavigableRedirect(''), false)
  assert.equal(isNavigableRedirect('   '), false)
  assert.equal(isNavigableRedirect(undefined), false)
  assert.equal(isNavigableRedirect(null), false)
})
