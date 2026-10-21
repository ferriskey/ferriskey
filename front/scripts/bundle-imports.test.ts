import assert from 'node:assert/strict'
import { test } from 'node:test'
import { findBareImports } from './bundle-imports.ts'

test('a default import of a package name is bare', () => {
  assert.deepEqual(findBareImports('import oOe from"cheerio";function f(){}'), ['cheerio'])
})

test('named, namespace and side-effect imports are all read', () => {
  const source = 'import{a as b,c}from"juice";import*as n from"@scope/pkg";import"side-effect";var x=1'
  assert.deepEqual(findBareImports(source), ['juice', '@scope/pkg', 'side-effect'])
})

test('re-exports from a package name are bare', () => {
  assert.deepEqual(findBareImports('export*from"left-pad";export{a}from"right-pad";'), [
    'left-pad',
    'right-pad',
  ])
})

test('relative, absolute and url specifiers resolve in a browser', () => {
  const source =
    'import{a}from"./chunk-1.js";import b from"../shared.js";import"/assets/x.js";import c from"https://cdn.example/c.js";'
  assert.deepEqual(findBareImports(source), [])
})

test('leading comments and whitespace do not hide an import', () => {
  const source = '/*! license */\n// banner\nimport x from"cheerio";\nimport y from"./y.js";'
  assert.deepEqual(findBareImports(source), ['cheerio'])
})

test('import-looking text inside code is not an import', () => {
  const source = 'import a from"./a.js";var s=";import x from\\"cheerio\\"";console.log(s)'
  assert.deepEqual(findBareImports(source), [])
})

test('a module without imports has none', () => {
  assert.deepEqual(findBareImports('export{a as default};'), [])
})
