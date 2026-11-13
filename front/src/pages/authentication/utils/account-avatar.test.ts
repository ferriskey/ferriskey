import test from 'node:test'
import assert from 'node:assert/strict'

import { avatarColorClass, avatarInitial } from './account-avatar.ts'

const PALETTE = [
  'border-fk-primary-border bg-fk-primary-soft text-fk-primary-text',
  'border-fk-info-border bg-fk-info-soft text-fk-info',
  'border-fk-success-border bg-fk-success-soft text-fk-success',
  'border-fk-violet-border bg-fk-violet-soft text-fk-violet',
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
