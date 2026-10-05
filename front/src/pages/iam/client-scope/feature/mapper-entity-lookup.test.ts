import test from 'node:test'
import assert from 'node:assert/strict'

import {
  pickSelectedRole,
  roleValue,
  selectedClientQuery,
  selectedRoleQuery,
} from './mapper-entity-lookup.ts'

const realmRole = (name: string) => ({ name, client: null })
const clientRole = (clientId: string, name: string) => ({ name, client: { client_id: clientId } })

test('a saved client is resolved by its exact client id, one row at most', () => {
  assert.deepEqual(selectedClientQuery('app-web'), { client_id_exact: 'app-web', limit: 1 })
})

test('a saved role is resolved by its exact qualified name, both colliding roles fetched', () => {
  assert.deepEqual(selectedRoleQuery('app-web.ns.admin'), {
    qualified_name: 'app-web.ns.admin',
    limit: 2,
  })
})

test('the role value qualifies client roles with their client id', () => {
  assert.equal(roleValue(realmRole('admin')), 'admin')
  assert.equal(roleValue(clientRole('app-web', 'ns.admin')), 'app-web.ns.admin')
})

test('the exact role is picked among the returned candidates', () => {
  const realm = realmRole('a.b')
  const client = clientRole('a', 'b')
  assert.equal(pickSelectedRole([client, realm], 'a.b'), client)
  assert.equal(pickSelectedRole([realmRole('a.bc')], 'a.b'), undefined)
  assert.equal(pickSelectedRole(undefined, 'a.b'), undefined)
})
