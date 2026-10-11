import assert from 'node:assert/strict'
import { describe, it } from 'node:test'
import {
  ownerClientOptions,
  ownersChanged,
  ownersFromList,
  ownersToList,
} from './resource-owners.ts'

const client = (id: string, overrides: Record<string, unknown> = {}) => ({
  id,
  client_id: `cid-${id}`,
  name: `Client ${id}`,
  public_client: false,
  registration_source: 'admin',
  ...overrides,
})

describe('ownerClientOptions', () => {
  it('keeps only confidential admin-registered clients', () => {
    const options = ownerClientOptions([
      client('a'),
      client('b', { public_client: true }),
      client('c', { registration_source: 'dynamic' }),
      client('d', { registration_source: 'metadata_document' }),
    ])
    assert.deepEqual(
      options.map((option) => option.id),
      ['a']
    )
  })

  it('falls back to the client_id when the name is empty', () => {
    const [option] = ownerClientOptions([client('a', { name: '' })])
    assert.equal(option.label, 'cid-a')
  })
})

describe('ownersFromList / ownersToList', () => {
  it('keeps owners only for the resources still listed', () => {
    const record = ownersFromList([
      { uri: 'https://a.example', client_id: '1' },
      { uri: 'https://b.example', client_id: '2' },
    ])
    assert.deepEqual(ownersToList(record, ['https://b.example']), [
      { uri: 'https://b.example', client_id: '2' },
    ])
  })

  it('drops unset owners', () => {
    assert.deepEqual(ownersToList({ 'https://a.example': '' }, ['https://a.example']), [])
  })
})

describe('ownersChanged', () => {
  const pristine = { 'https://a.example': '1' }
  const listed = ['https://a.example']

  it('is false when nothing differs', () => {
    assert.equal(ownersChanged(pristine, { 'https://a.example': '1' }, listed), false)
  })

  it('detects a changed owner', () => {
    assert.equal(ownersChanged(pristine, { 'https://a.example': '2' }, listed), true)
  })

  it('detects a cleared owner', () => {
    assert.equal(ownersChanged(pristine, { 'https://a.example': '' }, listed), true)
  })

  it('ignores owners of removed resources', () => {
    assert.equal(ownersChanged(pristine, pristine, []), false)
  })
})
