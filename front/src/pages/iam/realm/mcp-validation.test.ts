import assert from 'node:assert/strict'
import { test } from 'node:test'
import {
  invalidEntries,
  isValidCimdHost,
  isValidResource,
  validateEntry,
} from './mcp-validation.ts'

test('accepts https resources', () => {
  assert.equal(isValidResource('https://mcp.example.com/api'), true)
})

test('accepts http on loopback only', () => {
  assert.equal(isValidResource('http://localhost:8080/mcp'), true)
  assert.equal(isValidResource('http://127.0.0.1/mcp'), true)
  assert.equal(isValidResource('http://[::1]:3000/mcp'), true)
  assert.equal(isValidResource('http://example.com/mcp'), false)
})

test('rejects fragments, relative and other schemes', () => {
  assert.equal(isValidResource('https://example.com/a#frag'), false)
  assert.equal(isValidResource('/relative'), false)
  assert.equal(isValidResource('ftp://example.com'), false)
  assert.equal(isValidResource(' https://example.com'), false)
  assert.equal(isValidResource(''), false)
})

test('accepts bare hosts', () => {
  assert.equal(isValidCimdHost('example.com'), true)
  assert.equal(isValidCimdHost('client.example.com:8443'), true)
  assert.equal(isValidCimdHost('localhost'), true)
})

test('rejects hosts with scheme, path or spaces', () => {
  assert.equal(isValidCimdHost('https://example.com'), false)
  assert.equal(isValidCimdHost('example.com/path'), false)
  assert.equal(isValidCimdHost('exa mple.com'), false)
  assert.equal(isValidCimdHost(''), false)
})

test('validateEntry flags invalid and duplicate entries', () => {
  assert.equal(validateEntry('a.com', [], isValidCimdHost), null)
  assert.equal(validateEntry('a b', [], isValidCimdHost), 'invalid')
  assert.equal(validateEntry('a.com', ['a.com'], isValidCimdHost), 'duplicate')
})

test('invalidEntries lists the failing values', () => {
  assert.deepEqual(invalidEntries(['a.com', 'x/y'], isValidCimdHost), ['x/y'])
})
