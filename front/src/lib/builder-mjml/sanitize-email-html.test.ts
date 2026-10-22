import test from 'node:test'
import assert from 'node:assert/strict'

import { JSDOM } from 'jsdom'

import { createEmailHtmlSanitizer } from './sanitize-email-html.ts'

const sanitize = createEmailHtmlSanitizer(new JSDOM('').window)

test('text formatting survives', () => {
  const html = '<p><strong>Hello</strong> <em>there</em> <u>you</u></p><ul><li>one</li></ul>'

  assert.equal(sanitize(html), html)
})

test('template variables are left alone in text and in links', () => {
  const html = '<p>Hi {{ firstname }}, <a href="{{ reset_link }}">reset</a></p>'

  assert.equal(sanitize(html), html)
})

test('links to http, https and mailto keep their target', () => {
  for (const href of ['https://example.com/a?b=1', 'http://example.com', 'mailto:a@example.com']) {
    assert.match(sanitize(`<a href="${href}">x</a>`), new RegExp(`href="${href.replace(/[.?]/g, '\\$&')}"`))
  }
})

test('a javascript link loses its target', () => {
  assert.doesNotMatch(sanitize('<a href="javascript:alert(1)">x</a>'), /javascript/i)
  assert.doesNotMatch(sanitize('<a href="  JaVaScRiPt:alert(1)">x</a>'), /javascript/i)
  assert.doesNotMatch(sanitize('<a href="data:text/html,<script>alert(1)</script>">x</a>'), /data:/i)
})

test('event handlers and scripts are removed', () => {
  assert.doesNotMatch(sanitize('<p onclick="alert(1)">x</p>'), /onclick|alert/)
  assert.doesNotMatch(sanitize('<img src="x" onerror="alert(1)">'), /<img|onerror|alert/)
  assert.doesNotMatch(sanitize('<script>alert(1)</script><p>x</p>'), /script|alert/)
  assert.doesNotMatch(sanitize('<a href="https://example.com" onmouseover="alert(1)">x</a>'), /onmouseover|alert/)
})

test('elements that can run or load code are removed', () => {
  for (const html of [
    '<svg onload="alert(1)"></svg>',
    '<math><mi xlink:href="javascript:alert(1)">x</mi></math>',
    '<iframe src="https://example.com"></iframe>',
    '<form action="https://example.com"><input></form>',
    '<object data="x"></object>',
    '<embed src="x">',
    '<link rel="stylesheet" href="https://example.com/x.css">',
    '<meta http-equiv="refresh" content="0;url=https://example.com">',
    '<base href="https://example.com/">',
  ]) {
    assert.equal(sanitize(html), '', html)
  }
})

test('inline styles keep presentation and lose anything that loads a resource', () => {
  const kept = sanitize('<p style="color: #ff0000; text-align: center">x</p>')
  assert.match(kept, /color: ?#ff0000/)
  assert.match(kept, /text-align: ?center/)

  assert.doesNotMatch(sanitize('<p style="background: url(https://evil.example/x)">x</p>'), /url|evil/)
  assert.doesNotMatch(sanitize('<p style="width: expression(alert(1))">x</p>'), /expression|alert/)
  assert.doesNotMatch(sanitize('<p style="position: fixed; top: 0">x</p>'), /position/)
})

test('an unclosed tag is closed instead of swallowing what follows', () => {
  assert.equal(sanitize('<p><strong>x'), '<p><strong>x</strong></p>')
})
