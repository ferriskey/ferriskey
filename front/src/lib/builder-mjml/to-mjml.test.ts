import test from 'node:test'
import assert from 'node:assert/strict'

import { JSDOM } from 'jsdom'

import type { BuilderNode } from '../builder-core/types.ts'
import { createEmailHtmlSanitizer } from './sanitize-email-html.ts'
import { treeToMjml } from './to-mjml.ts'

const sanitize = createEmailHtmlSanitizer(new JSDOM('').window)

function node(overrides: Partial<BuilderNode>): BuilderNode {
  return { id: 'n', type: 'mj-text', props: {}, styles: {}, children: [], ...overrides }
}

test('attribute values cannot close their attribute', () => {
  const mjml = treeToMjml([node({ props: { padding: '10px" onload="alert(1)' } })], sanitize)

  assert.doesNotMatch(mjml, /padding="10px" onload/)
  assert.match(mjml, /padding="10px&quot; onload=&quot;alert\(1\)"/)
})

test('an attribute name that is not a plain name is dropped', () => {
  const mjml = treeToMjml([node({ props: { 'x" onload="y': 'v', 'font-size': '12px' } })], sanitize)

  assert.doesNotMatch(mjml, /onload/)
  assert.match(mjml, /font-size="12px"/)
})

test('the content of a text block is sanitized', () => {
  const mjml = treeToMjml(
    [node({ content: '<p>Hello</p><img src="x" onerror="alert(1)"><script>alert(2)</script>' })],
    sanitize
  )

  assert.match(mjml, /<mj-text><p>Hello<\/p><\/mj-text>/)
  assert.doesNotMatch(mjml, /img|onerror|script|alert/)
})

test('the content of a button is sanitized', () => {
  const mjml = treeToMjml(
    [node({ type: 'mj-button', content: 'Go <img src="x" onerror="alert(1)">' })],
    sanitize
  )

  assert.doesNotMatch(mjml, /img|onerror|alert/)
})

test('markup in the content cannot close its block', () => {
  const mjml = treeToMjml(
    [node({ content: '</mj-text></mj-column><mj-raw><script>alert(1)</script></mj-raw>' })],
    sanitize
  )

  assert.equal(mjml, '<mjml><mj-body><mj-text></mj-text></mj-body></mjml>')
})

test('blocks without content keep their shape', () => {
  const mjml = treeToMjml(
    [node({ type: 'mj-divider', props: { 'border-color': '#000' } }), node({ type: 'mj-spacer' })],
    sanitize
  )

  assert.equal(
    mjml,
    '<mjml><mj-body><mj-divider border-color="#000" /><mj-spacer /></mj-body></mjml>'
  )
})
