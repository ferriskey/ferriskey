import test from 'node:test'
import assert from 'node:assert/strict'

// Node has no Web Storage: in-memory shims installed before importing the module under test.
const memoryStorage = (): Storage => {
  const entries = new Map<string, string>()
  return {
    getItem: (key: string) => entries.get(key) ?? null,
    setItem: (key: string, value: string) => void entries.set(key, value),
    removeItem: (key: string) => void entries.delete(key),
    clear: () => entries.clear(),
    key: (index: number) => [...entries.keys()][index] ?? null,
    get length() {
      return entries.size
    },
  }
}
globalThis.sessionStorage = memoryStorage()
globalThis.localStorage = memoryStorage()

const { buildProviderLoginUrl } = await import('./login-providers-helpers.ts')
const { deriveCodeChallenge, takeOAuthFlow } = await import('../utils/pkce.ts')
const { validateCallbackParams } = await import('../feature/callback-helpers.ts')

const BASE_INPUT = {
  loginUrl: '/realms/master/broker/github/login',
  apiUrl: 'https://api.ferriskey.test',
  currentSearch: '',
  origin: 'https://console.ferriskey.test',
  pathname: '/realms/master/authentication/login',
}

test('resolves a relative login_url against the api base', async () => {
  const url = new URL(await buildProviderLoginUrl(BASE_INPUT))
  assert.equal(url.origin + url.pathname, 'https://api.ferriskey.test/realms/master/broker/github/login')
})

test('keeps an absolute login_url as-is', async () => {
  const url = new URL(
    await buildProviderLoginUrl({
      ...BASE_INPUT,
      loginUrl: 'https://idp.example.com/authorize',
    })
  )
  assert.equal(url.origin, 'https://idp.example.com')
})

test('starts a full authorization code flow with PKCE S256', async () => {
  const url = new URL(await buildProviderLoginUrl(BASE_INPUT))
  assert.equal(url.searchParams.get('response_type'), 'code')
  assert.equal(url.searchParams.get('client_id'), 'security-admin-console')
  assert.equal(url.searchParams.get('code_challenge_method'), 'S256')
  const challenge = url.searchParams.get('code_challenge')
  assert.ok(challenge && challenge.length === 43, 'S256 challenge is 32 bytes base64url-encoded')
  assert.match(challenge, /^[A-Za-z0-9_-]+$/)
})

test('defaults redirect_uri to the console callback of the current realm', async () => {
  const url = new URL(
    await buildProviderLoginUrl({
      ...BASE_INPUT,
      pathname: '/realms/acme/authentication/login',
    })
  )
  assert.equal(
    url.searchParams.get('redirect_uri'),
    'https://console.ferriskey.test/realms/acme/authentication/callback'
  )
})

test('falls back to the master realm outside a realm path', async () => {
  const url = new URL(
    await buildProviderLoginUrl({ ...BASE_INPUT, pathname: '/authentication/login' })
  )
  assert.equal(
    url.searchParams.get('redirect_uri'),
    'https://console.ferriskey.test/realms/master/authentication/callback'
  )
})

test('stores a single-use verifier matching the challenge under the state', async () => {
  const url = new URL(await buildProviderLoginUrl(BASE_INPUT))
  const state = url.searchParams.get('state')
  assert.ok(state)

  const flow = takeOAuthFlow(state)
  assert.ok(flow)
  assert.equal(flow.redirectUri, url.searchParams.get('redirect_uri'))
  assert.equal(await deriveCodeChallenge(flow.codeVerifier), url.searchParams.get('code_challenge'))

  // Single-use: a second read returns null.
  assert.equal(takeOAuthFlow(state), null)
})

test('registers the state so the callback page accepts it', async () => {
  const url = new URL(await buildProviderLoginUrl(BASE_INPUT))
  const state = url.searchParams.get('state')

  assert.equal(
    validateCallbackParams({
      code: 'code-1',
      returnedState: state,
      expectedState: localStorage.getItem(`oauth_state:${state}`),
    }),
    null
  )
})

test('rejects without registering a flow when the challenge cannot be derived', async (t) => {
  t.mock.method(crypto.subtle, 'digest', () =>
    Promise.reject(new TypeError('crypto.subtle is unavailable'))
  )
  const stateSpy = t.mock.method(crypto, 'randomUUID', () => 'state-unavailable')

  await assert.rejects(buildProviderLoginUrl(BASE_INPUT), TypeError)
  assert.equal(stateSpy.mock.callCount(), 1)
  assert.equal(localStorage.getItem('oauth_state:state-unavailable'), null)
  assert.equal(takeOAuthFlow('state-unavailable'), null)
})

test('forwards an application flow untouched, state included', async () => {
  const storedBefore = localStorage.length
  const url = new URL(
    await buildProviderLoginUrl({
      ...BASE_INPUT,
      currentSearch: '?client_id=my-app&redirect_uri=https://app.example.com/cb&state=app-state',
    })
  )
  assert.equal(url.searchParams.get('client_id'), 'my-app')
  assert.equal(url.searchParams.get('redirect_uri'), 'https://app.example.com/cb')
  assert.equal(url.searchParams.get('state'), 'app-state')
  assert.equal(url.searchParams.get('code_challenge'), null)
  assert.equal(takeOAuthFlow('app-state'), null)
  assert.equal(localStorage.length, storedBefore, 'no console state registered for an application')
})

test('replaces a console state carried over from /authorize with a registered one', async () => {
  const url = new URL(
    await buildProviderLoginUrl({
      ...BASE_INPUT,
      currentSearch: '?client_id=security-admin-console&state=stale&session_id=00000000-0000-0000-0000-000000000000',
    })
  )
  const state = url.searchParams.get('state')
  assert.notEqual(state, 'stale')
  assert.equal(localStorage.getItem(`oauth_state:${state}`), state)
  assert.equal(url.searchParams.get('session_id'), null)
})

test('ignores a console redirect_uri carried over from another realm', async () => {
  const url = new URL(
    await buildProviderLoginUrl({
      ...BASE_INPUT,
      pathname: '/realms/acme/authentication/login',
      currentSearch:
        '?client_id=security-admin-console&redirect_uri=https://console.ferriskey.test/realms/master/authentication/callback',
    })
  )
  assert.equal(
    url.searchParams.get('redirect_uri'),
    'https://console.ferriskey.test/realms/acme/authentication/callback'
  )
})

test('forwards the current query params unless already present', async () => {
  const url = new URL(
    await buildProviderLoginUrl({
      ...BASE_INPUT,
      loginUrl: '/realms/master/broker/github/login?prompt=login',
      currentSearch: '?prompt=consent&kc_locale=fr',
    })
  )
  assert.equal(url.searchParams.get('prompt'), 'login')
  assert.equal(url.searchParams.get('kc_locale'), 'fr')
})

test('two clicks produce two independent flows', async () => {
  const first = new URL(await buildProviderLoginUrl(BASE_INPUT))
  const second = new URL(await buildProviderLoginUrl(BASE_INPUT))
  assert.notEqual(first.searchParams.get('state'), second.searchParams.get('state'))
  assert.notEqual(
    first.searchParams.get('code_challenge'),
    second.searchParams.get('code_challenge')
  )
})
