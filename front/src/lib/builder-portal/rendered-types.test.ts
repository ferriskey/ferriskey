import test from 'node:test'
import assert from 'node:assert/strict'

import { collectRenderedTypes } from './rendered-types.ts'

function typesOf(tree: unknown): string[] {
  const acc = new Set<string>()
  collectRenderedTypes(tree, acc)
  return [...acc].sort()
}

test('counts blocks nested through every child-rendering node', () => {
  for (const parent of [
    'container',
    'flex',
    'grid',
    'div',
    'card',
    'card-header',
    'card-content',
    'card-footer',
  ]) {
    const tree = [{ type: parent, children: [{ type: 'account_list' }] }]
    assert.deepEqual(typesOf(tree), ['account_list', parent].sort(), parent)
  }
})

test('ignores children of a node the renderer draws without its children', () => {
  const tree = [
    { type: 'card', children: [{ type: 'text', children: [{ type: 'account_list' }] }] },
  ]
  assert.deepEqual(typesOf(tree), ['card', 'text'])
})

test('ignores a type that only appears inside props', () => {
  const tree = [{ type: 'text', props: { type: 'account_list' } }]
  assert.deepEqual(typesOf(tree), ['text'])
})

test('walks a root object without a type through its children', () => {
  assert.deepEqual(typesOf({ children: [{ type: 'account_list' }] }), ['account_list'])
})

test('tolerates empty and malformed input', () => {
  assert.deepEqual(typesOf(undefined), [])
  assert.deepEqual(typesOf(null), [])
  assert.deepEqual(typesOf([null, 3, 'x', { type: 4 }]), [])
})
