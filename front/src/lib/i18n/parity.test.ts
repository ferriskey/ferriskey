import test from 'node:test'
import assert from 'node:assert/strict'

import { checkParity, flattenCatalog, splitPluralKey, type CatalogSet } from './parity.ts'

function catalogSet(en: Record<string, string>, zh: Record<string, string>): CatalogSet {
  return { en: { common: en }, 'zh-CN': { common: zh } }
}

test('a nested catalogue flattens to dotted keys', () => {
  assert.deepEqual(flattenCatalog({ detail: { settings: { title: 'Client settings' } } }), {
    'detail.settings.title': 'Client settings',
  })
})

test('a plural suffix is split off the base key', () => {
  assert.deepEqual(splitPluralKey('list.count_one'), { base: 'list.count', category: 'one' })
  assert.equal(splitPluralKey('list.count'), null)
})

test('a plural declared with both english categories is not reported against chinese', () => {
  const findings = checkParity(
    catalogSet(
      { 'list.count_one': '{{count}} client', 'list.count_other': '{{count}} clients' },
      { 'list.count_other': '{{count}} 个客户端' }
    )
  )

  assert.deepEqual(findings, [])
})

test('a key present only in a target locale fails, because the source is never exempt', () => {
  const findings = checkParity(catalogSet({}, { 'detail.title': '客户端设置' }))

  assert.equal(findings.length, 1)
  assert.equal(findings[0].severity, 'error')
  assert.equal(findings[0].locale, 'en')
  assert.equal(findings[0].key, 'detail.title')
})

test('an untranslated catalogue is reported without failing the job', () => {
  const findings = checkParity(catalogSet({ 'detail.title': 'Client settings' }, {}))

  assert.equal(findings.length, 1)
  assert.equal(findings[0].severity, 'warning')
  assert.equal(findings[0].locale, 'zh-CN')
  assert.match(findings[0].message, /translation has not started/)
})

test('a catalogue that has started is held to every key, with no flag to flip', () => {
  const findings = checkParity(
    catalogSet(
      { 'detail.title': 'Client settings', 'detail.secret': 'Reveal secret' },
      { 'detail.title': '客户端设置' }
    )
  )

  assert.equal(findings.length, 1)
  assert.equal(findings[0].severity, 'error')
  assert.equal(findings[0].locale, 'zh-CN')
  assert.equal(findings[0].key, 'detail.secret')
})

test('emptiness is judged per namespace, so one translated namespace does not bind the others', () => {
  const findings = checkParity({
    en: { common: { 'a.b': 'A' }, client: { 'c.d': 'C' } },
    'zh-CN': { common: { 'a.b': '甲' }, client: {} },
  })

  assert.equal(findings.length, 1)
  assert.equal(findings[0].severity, 'warning')
  assert.equal(findings[0].namespace, 'client')
})

test('a chinese catalogue carrying an english-only plural form is an error', () => {
  const findings = checkParity(
    catalogSet(
      { 'list.count_one': '{{count}} client', 'list.count_other': '{{count}} clients' },
      { 'list.count_one': '一个客户端', 'list.count_other': '{{count}} 个客户端' }
    )
  )

  assert.equal(findings.length, 1)
  assert.equal(findings[0].severity, 'error')
  assert.equal(findings[0].locale, 'zh-CN')
  assert.equal(findings[0].key, 'list.count_one')
})

test('a chinese plural missing its only category is an error, not a warning', () => {
  const findings = checkParity(
    catalogSet(
      { 'list.count_one': '{{count}} client', 'list.count_other': '{{count}} clients' },
      { 'list.count_one': '一个客户端' }
    )
  )

  const messages = findings.map((finding) => `${finding.severity} ${finding.locale} ${finding.key}`)

  assert.deepEqual(messages.sort(), ['error zh-CN list.count_one', 'error zh-CN list.count_other'])
})

test('an english plural missing a category it declares is an error', () => {
  const findings = checkParity(
    catalogSet({ 'list.count_other': '{{count}} clients' }, { 'list.count_other': '{{count}} 个' })
  )

  assert.equal(findings.length, 1)
  assert.equal(findings[0].severity, 'error')
  assert.equal(findings[0].locale, 'en')
  assert.equal(findings[0].key, 'list.count_one')
})

test('a key whose last segment merely looks plural is compared literally', () => {
  const findings = checkParity(
    catalogSet(
      { 'policy.require_one': 'Require at least one' },
      { 'policy.require_one': '至少需要一个' }
    )
  )

  assert.deepEqual(findings, [])
})
