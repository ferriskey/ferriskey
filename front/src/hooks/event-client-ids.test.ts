import test from 'node:test'
import assert from 'node:assert/strict'

import { eventClientIds } from './event-client-ids.ts'

const C1 = '0199a4f2-3c1e-7b8a-9f00-000000000001'
const C2 = '0199a4f2-3c1e-7b8a-9f00-000000000002'
const X1 = '0199a4f2-3c1e-7b8a-9f00-000000000003'

const event = (
  actor: [string | null, string | null],
  target: [string | null, string | null]
) => ({
  actor_id: actor[0],
  actor_type: actor[1],
  target_id: target[0],
  target_type: target[1],
})

test('client targets and actors are collected', () => {
  assert.deepEqual(eventClientIds([event([C1, 'client'], [C2, 'client'])]), [C1, C2])
})

test('users, service accounts, admins and roles are not looked up as clients', () => {
  assert.deepEqual(
    eventClientIds([
      event([C1, 'user'], [C2, 'role']),
      event([X1, 'service_account'], [C1, 'admin']),
    ]),
    []
  )
})

test('ids of an unknown kind stay resolvable as clients', () => {
  assert.deepEqual(eventClientIds([event([null, null], [X1, null])]), [X1])
})

test('values that are not uuids are left out of the lookup', () => {
  assert.deepEqual(
    eventClientIds([event(['admin-cli', 'client'], [C1, 'client'])]),
    [C1]
  )
})
