import test from 'node:test'
import assert from 'node:assert/strict'

import { DERIVED_ORIGIN_SENTINEL, isWebOriginValue, normalizeWebOrigin } from './web-origin.ts'

test('an origin keeps its scheme, host and non-default port', () => {
  assert.equal(
    normalizeWebOrigin('https://app.example.com:8443'),
    'https://app.example.com:8443'
  )
})

test('a default port is dropped, as the browser drops it', () => {
  assert.equal(normalizeWebOrigin('https://app.example.com:443'), 'https://app.example.com')
  assert.equal(normalizeWebOrigin('http://app.example.com:80'), 'http://app.example.com')
})

test('scheme and host are lowercased', () => {
  assert.equal(normalizeWebOrigin('HTTPS://App.Example.COM'), 'https://app.example.com')
})

test('a lone trailing slash is not a path', () => {
  assert.equal(normalizeWebOrigin('https://app.example.com/'), 'https://app.example.com')
})

test('surrounding whitespace is trimmed', () => {
  assert.equal(normalizeWebOrigin('  https://app.example.com  '), 'https://app.example.com')
})

test('an origin carries no path', () => {
  assert.equal(normalizeWebOrigin('https://app.example.com/callback'), null)
})

test('an origin carries no query or fragment', () => {
  assert.equal(normalizeWebOrigin('https://app.example.com/?next=1'), null)
  assert.equal(normalizeWebOrigin('https://app.example.com/#top'), null)
})

test('an origin carries no credentials', () => {
  assert.equal(normalizeWebOrigin('https://user:secret@app.example.com'), null)
})

test('the wildcard is refused, alone or in a host', () => {
  assert.equal(normalizeWebOrigin('*'), null)
  assert.equal(normalizeWebOrigin('  *  '), null)
  assert.equal(normalizeWebOrigin('https://*.example.com'), null)
})

test('only http and https have an origin we can register', () => {
  assert.equal(normalizeWebOrigin('ftp://files.example.com'), null)
  assert.equal(normalizeWebOrigin('file:///etc/passwd'), null)
  assert.equal(normalizeWebOrigin('chrome-extension://abcdef'), null)
})

test('what is not a URL at all is refused', () => {
  assert.equal(normalizeWebOrigin(''), null)
  assert.equal(normalizeWebOrigin('null'), null)
  assert.equal(normalizeWebOrigin('not an origin'), null)
})

test('the sentinel is not an origin, so it does not normalize', () => {
  assert.equal(normalizeWebOrigin(DERIVED_ORIGIN_SENTINEL), null)
})

test('the sentinel is nonetheless an accepted value', () => {
  assert.equal(isWebOriginValue(DERIVED_ORIGIN_SENTINEL), true)
  assert.equal(isWebOriginValue('  +  '), true)
})

test('an accepted value is either the sentinel or a valid origin', () => {
  assert.equal(isWebOriginValue('https://app.example.com'), true)
  assert.equal(isWebOriginValue('https://app.example.com/callback'), false)
  assert.equal(isWebOriginValue('*'), false)
  assert.equal(isWebOriginValue(''), false)
})
