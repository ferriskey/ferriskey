import test from 'node:test'
import assert from 'node:assert/strict'

function memoryStorage(): Storage {
  const data = new Map<string, string>()
  return {
    get length() {
      return data.size
    },
    clear: () => data.clear(),
    getItem: (key) => data.get(key) ?? null,
    key: (index) => [...data.keys()][index] ?? null,
    removeItem: (key) => void data.delete(key),
    setItem: (key, value) => void data.set(key, String(value)),
  }
}

const session = memoryStorage()
const local = memoryStorage()
globalThis.sessionStorage = session
globalThis.localStorage = local

const { authStore } = await import('./auth.store.ts')

test('tokens are kept in the session storage of the tab, never in local storage', () => {
  authStore.getState().setTokens('access', 'refresh', 'id')
  authStore.getState().setSessionRealm('master')

  const persisted = JSON.parse(session.getItem('auth') ?? '{}')
  assert.equal(persisted.state.refreshToken, 'refresh')
  assert.equal(persisted.state.accessToken, 'access')
  assert.equal(persisted.state.sessionRealm, 'master')
  assert.equal(local.getItem('auth'), null)
  assert.equal(local.length, 0)
})

test('clearing the tokens leaves nothing usable in storage', () => {
  authStore.getState().setTokens('access', 'refresh', 'id')
  authStore.getState().setTokens(null, null, null)

  const persisted = JSON.parse(session.getItem('auth') ?? '{}')
  assert.equal(persisted.state.accessToken, null)
  assert.equal(persisted.state.refreshToken, null)
  assert.equal(persisted.state.idToken, null)
})
