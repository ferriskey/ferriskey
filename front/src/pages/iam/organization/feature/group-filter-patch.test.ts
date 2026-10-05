import test from 'node:test'
import assert from 'node:assert/strict'

import { groupFilterPatch, walkDownPatch } from './group-filter-patch.ts'

test('picking a parent clears the top-level filter', () => {
  assert.deepEqual(groupFilterPatch('parent_group_id', 'g1'), {
    parent_group_id: 'g1',
    is_root: '',
  })
})

test('asking for top-level groups clears the parent filter', () => {
  assert.deepEqual(groupFilterPatch('is_root', 'true'), { is_root: 'true', parent_group_id: '' })
})

test('nested groups keep the parent filter', () => {
  assert.deepEqual(groupFilterPatch('is_root', 'false'), { is_root: 'false' })
})

test('clearing the parent leaves the other filters alone', () => {
  assert.deepEqual(groupFilterPatch('parent_group_id', ''), { parent_group_id: '' })
})

test('text filters pass through', () => {
  assert.deepEqual(groupFilterPatch('name', 'eng'), { name: 'eng' })
})

test('walking down sets the parent and clears top-level in one patch', () => {
  assert.deepEqual(walkDownPatch('g2'), { parent_group_id: 'g2', is_root: '' })
})
