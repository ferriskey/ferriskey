import test from 'node:test'
import assert from 'node:assert/strict'

import { avatarColorClass, avatarInitial } from './account-avatar.ts'

const PALETTE = [
  'bg-teal-600',
  'bg-purple-600',
  'bg-indigo-600',
  'bg-green-700',
  'bg-orange-600',
  'bg-blue-600',
]

test('the initial is the uppercased first character', () => {
  assert.equal(avatarInitial('alice'), 'A')
  assert.equal(avatarInitial('  bob'), 'B')
  assert.equal(avatarInitial('émile'), 'É')
})

test('the initial keeps astral characters whole', () => {
  assert.equal(avatarInitial('😀 smile'), '😀')
})

test('a blank name falls back to a question mark', () => {
  assert.equal(avatarInitial(''), '?')
  assert.equal(avatarInitial('   '), '?')
})

test('the colour is deterministic and within the palette', () => {
  const first = avatarColorClass('Alice', 'a***@x.io')
  assert.equal(first, avatarColorClass('Alice', 'a***@x.io'))
  assert.ok(PALETTE.includes(first))
})

test('a missing email is treated as empty', () => {
  assert.equal(avatarColorClass('Alice', null), avatarColorClass('Alice', ''))
})

test('different accounts spread over several tones', () => {
  const tones = new Set(
    ['Alice', 'Bob', 'Carol', 'Dave', 'Erin', 'Frank', 'Grace', 'Heidi'].map((n) =>
      avatarColorClass(n, null)
    )
  )
  assert.ok(tones.size > 1)
})
