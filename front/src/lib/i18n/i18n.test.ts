import test from 'node:test'
import assert from 'node:assert/strict'
import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { parse } from 'yaml'
import { createInstance } from 'i18next'

import { buildInitOptions, humanizeKey } from './config.ts'

function readCatalog(locale: string, namespace: string) {
  const path = fileURLToPath(new URL(`../../locales/${locale}/${namespace}.yaml`, import.meta.url))
  return parse(readFileSync(path, 'utf8'))
}

const resources = {
  en: { common: readCatalog('en', 'common') },
  'zh-CN': { common: readCatalog('zh-CN', 'common') },
}

async function instanceFor(lng: string) {
  const instance = createInstance()
  await instance.init(buildInitOptions({ lng, resources }))
  return instance
}

test('english plural selects the right form for 0, 1 and 2', async () => {
  const i18n = await instanceFor('en')

  assert.equal(i18n.t('relative_time.minutes', { count: 0 }), '0 min ago')
  assert.equal(i18n.t('relative_time.minutes', { count: 1 }), '1 min ago')
  assert.equal(i18n.t('relative_time.minutes', { count: 2 }), '2 min ago')
})

test('english resolves the plural categories intl declares for it', () => {
  assert.deepEqual(new Intl.PluralRules('en').resolvedOptions().pluralCategories.sort(), [
    'one',
    'other',
  ])
})

test('simplified chinese has a single plural category and still interpolates', async () => {
  const i18n = await instanceFor('zh-CN')

  assert.deepEqual(new Intl.PluralRules('zh-CN').resolvedOptions().pluralCategories, ['other'])
  assert.equal(i18n.t('relative_time.minutes', { count: 0 }), '0 分钟前')
  assert.equal(i18n.t('relative_time.minutes', { count: 1 }), '1 分钟前')
  assert.equal(i18n.t('relative_time.minutes', { count: 2 }), '2 分钟前')
})

test('a key missing from chinese falls back to the english value', async () => {
  const i18n = await instanceFor('zh-CN')

  assert.equal(i18n.t('language.names.en'), 'English')
  assert.equal(i18n.t('language.switcher_label'), '语言')
})

test('a key missing everywhere renders readable text and never the raw key', async () => {
  const i18n = await instanceFor('en')
  const warnings: string[] = []
  const originalWarn = console.warn
  console.warn = (...args: unknown[]) => void warnings.push(args.join(' '))

  try {
    const rendered = i18n.t('client:list.empty_state')

    assert.notEqual(rendered, 'client:list.empty_state')
    assert.notEqual(rendered, 'list.empty_state')
    assert.notEqual(rendered.trim(), '')
    assert.equal(rendered, 'Empty state')
    assert.ok(warnings.some((warning) => warning.includes('missing translation')))
  } finally {
    console.warn = originalWarn
  }
})

test('humanize turns a key into readable text', () => {
  assert.equal(humanizeKey('client:detail.settings.client_id'), 'Client id')
  assert.equal(humanizeKey('list.emptyState'), 'Empty state')
})
