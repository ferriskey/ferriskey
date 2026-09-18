import test from 'node:test'
import assert from 'node:assert/strict'
import { readdirSync } from 'node:fs'
import { fileURLToPath } from 'node:url'

const localesRoot = fileURLToPath(new URL('../../locales', import.meta.url))

function namespacesFor(locale: string): string[] {
  return readdirSync(`${localesRoot}/${locale}`)
    .filter((file) => file.endsWith('.yaml'))
    .map((file) => file.slice(0, -'.yaml'.length))
    .sort()
}

test('every english namespace exists on disk for the glob to bundle', () => {
  const english = namespacesFor('en')

  assert.ok(english.length > 1, 'the fallback locale ships more than one namespace')
  assert.ok(english.includes('common'), 'the default namespace is present')
})

test('the fallback locale covers every namespace the target locale declares', () => {
  const english = namespacesFor('en')
  const chinese = namespacesFor('zh-CN')

  const uncovered = chinese.filter((namespace) => !english.includes(namespace))

  assert.deepEqual(
    uncovered,
    [],
    'a namespace translated but absent from the fallback would render humanized keys'
  )
})
