import test from 'node:test'
import assert from 'node:assert/strict'
import { createHash } from 'node:crypto'
import { readFileSync } from 'node:fs'

const read = (name: string) => readFileSync(new URL(`../${name}`, import.meta.url), 'utf8')

const CONFIGS = ['nginx.conf']

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

test('the webapp image lets its entrypoint add the API origin to connect-src', () => {
  const policy = headerValue(read('nginx.conf'), 'Content-Security-Policy')
  assert.match(policy, /connect-src 'self' \$\{CSP_CONNECT_SRC\};/)
  assert.match(read('docker-entrypoint.sh'), /CSP_CONNECT_SRC/)
})

test('the console loads no font from a third party', () => {
  assert.ok(!read('index.html').includes('fonts.googleapis.com'))
  assert.ok(!read('index.html').includes('fonts.gstatic.com'))
})

for (const name of CONFIGS) {
  test(`${name} sends its security headers on every response`, () => {
    const config = read(name)

    assert.equal(headerValue(config, 'X-Content-Type-Options'), 'nosniff')
    assert.equal(headerValue(config, 'X-Frame-Options'), 'SAMEORIGIN')
    assert.equal(headerValue(config, 'Referrer-Policy'), 'no-referrer')
    assert.match(headerValue(config, 'Permissions-Policy'), /camera=\(\)/)
    assert.match(headerValue(config, 'Strict-Transport-Security'), /max-age=\d+/)
  })

  test(`${name} enforces a policy that only runs the console's own scripts`, () => {
    const policy = headerValue(read(name), 'Content-Security-Policy')
    const directives = Object.fromEntries(
      policy.split(';').map((directive) => {
        const [key, ...values] = directive.trim().split(/\s+/)
        return [key, values]
      })
    )

    assert.deepEqual(directives['script-src'], ['\'self\'', `'${inlineScriptHash()}'`])
    assert.deepEqual(directives['object-src'], ['\'none\''])
    assert.deepEqual(directives['base-uri'], ['\'self\''])
    assert.deepEqual(directives['frame-ancestors'], ['\'self\''])
    assert.deepEqual(directives['form-action'], ['\'self\''])
    assert.deepEqual(directives['default-src'], ['\'self\''])
    assert.ok(!policy.includes('\'unsafe-eval\''))
    assert.ok(!policy.includes('googleapis'), 'fonts are served by the console itself')
    assert.ok(!read(name).includes('Content-Security-Policy-Report-Only'), 'the policy is enforced')
  })

  test(`${name} hides the server version`, () => {
    assert.match(read(name), /server_tokens\s+off;/)
  })
}
