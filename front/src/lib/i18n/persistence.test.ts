import test from 'node:test'
import assert from 'node:assert/strict'

import { LOCALE_STORAGE_KEY } from './locales.ts'
import { readStoredLocale, writeStoredLocale } from './resolve-locale.ts'

function stubLocalStorage(initial: Record<string, string> = {}) {
  const store = new Map(Object.entries(initial))
  const storage = {
    getItem: (key: string) => store.get(key) ?? null,
    setItem: (key: string, value: string) => void store.set(key, value),
    removeItem: (key: string) => void store.delete(key),
    clear: () => store.clear(),
    key: (index: number) => [...store.keys()][index] ?? null,
    get length() {
      return store.size
    },
  }

  globalThis.window = { localStorage: storage } as unknown as Window & typeof globalThis
  return store
}

test('a chosen locale survives a reload through the storage mirror', () => {
  const store = stubLocalStorage()

  writeStoredLocale('zh-CN')

  assert.equal(store.get(LOCALE_STORAGE_KEY), 'zh-CN')
  assert.equal(readStoredLocale(), 'zh-CN')
})

test('an unsupported stored value resolves to nothing rather than being trusted', () => {
  stubLocalStorage({ [LOCALE_STORAGE_KEY]: 'fr-FR' })

  assert.equal(readStoredLocale(), null)
})

test('reading the mirror never throws when storage is unavailable', () => {
  globalThis.window = {
    get localStorage(): Storage {
      throw new Error('storage disabled')
    },
  } as unknown as Window & typeof globalThis

  assert.equal(readStoredLocale(), null)
  assert.doesNotThrow(() => writeStoredLocale('zh-CN'))
})
