import test from 'node:test'
import assert from 'node:assert/strict'

import { eventRoleIds } from './event-role-ids.ts'

const event = (
  actor: [string | null, string | null],
  target: [string | null, string | null]
) => ({
  actor_id: actor[0],
  actor_type: actor[1],
  target_id: target[0],
  target_type: target[1],
})

test('role targets and actors are collected', () => {
  assert.deepEqual(eventRoleIds([event(['r1', 'role'], ['r2', 'role'])]), ['r1', 'r2'])
})

test('users, service accounts, admins and clients are not looked up as roles', () => {
  assert.deepEqual(
    eventRoleIds([
      event(['u1', 'user'], ['c1', 'client']),
      event(['s1', 'service_account'], ['a1', 'admin']),
    ]),
    []
  )
})

test('ids of an unknown kind stay resolvable as roles', () => {
  assert.deepEqual(eventRoleIds([event([null, null], ['x1', null])]), ['x1'])
})
