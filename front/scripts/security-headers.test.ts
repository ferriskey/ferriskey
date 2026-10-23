import test from 'node:test'
import assert from 'node:assert/strict'
import { createHash } from 'node:crypto'
import { readFileSync } from 'node:fs'

const read = (name: string) => readFileSync(new URL(`../${name}`, import.meta.url), 'utf8')

const CONFIGS = ['nginx.conf', 'nginx-standalone.conf']

function inlineScriptHash(): string {
  const match = read('index.html').match(/<script>([\s\S]*?)<\/script>/)
  assert.ok(match, 'index.html carries one inline script')
  return `sha256-${createHash('sha256').update(match[1]).digest('base64')}`
}

function headerValue(config: string, name: string): string {
  const match = config.match(new RegExp(`add_header\\s+${name}\\s+(?:"([^"]*)"|'([^']*)'|(\\S+))\\s+always;`))
  assert.ok(match, `${name} is sent with always`)
  return match[1] ?? match[2] ?? match[3]
}

for (const name of CONFIGS) {
  test(`${name} sends its security headers on every response`, () => {
    const config = read(name)

    assert.equal(headerValue(config, 'X-Content-Type-Options'), 'nosniff')
    assert.equal(headerValue(config, 'X-Frame-Options'), 'SAMEORIGIN')
    assert.equal(headerValue(config, 'Referrer-Policy'), 'no-referrer')
    assert.match(headerValue(config, 'Permissions-Policy'), /camera=\(\)/)
    assert.match(headerValue(config, 'Strict-Transport-Security'), /max-age=\d+/)
  })

  test(`${name} enforces the directives that cannot break the console`, () => {
    const policy = headerValue(read(name), 'Content-Security-Policy')

    for (const directive of ['object-src \'none\'', 'base-uri \'self\'', 'frame-ancestors \'self\'']) {
      assert.ok(policy.includes(directive), directive)
    }
  })

  test(`${name} reports scripts that the strict policy would block`, () => {
    const policy = headerValue(read(name), 'Content-Security-Policy-Report-Only')

    assert.ok(policy.includes(`script-src 'self' '${inlineScriptHash()}'`), 'inline script hash')
    assert.ok(!policy.includes('\'unsafe-inline\'') || !/script-src[^;]*'unsafe-inline'/.test(policy))
    assert.ok(!/script-src[^;]*'unsafe-eval'/.test(policy))
  })

  test(`${name} hides the server version`, () => {
    assert.match(read(name), /server_tokens\s+off;/)
  })
}
