export type SelectOutcome =
  | { kind: 'redirect'; url: string }
  | { kind: 'login'; loginHint: string }
  | { kind: 'required-action'; action: string }
  | { kind: 'failed' }

interface SelectResponse {
  status?: string
  url?: string | null
  login_hint?: string
  required_actions?: readonly string[] | null
}

export function parseMaxAge(search: string): number | undefined {
  const raw = new URLSearchParams(search).get('max_age')
  if (raw === null || !/^\d+$/.test(raw)) return undefined
  const value = Number(raw)
  return value > 0 ? value : undefined
}

export function buildLoginPath(realm: string, search: string, loginHint?: string): string {
  const params = new URLSearchParams(search)
  params.delete('max_age')
  if (loginHint) params.set('login_hint', loginHint)
  const query = params.toString().replace(/\+/g, '%20')
  const base = `/realms/${realm}/authentication/login`
  return query ? `${base}?${query}` : base
}

export function resolveSelectOutcome(response: SelectResponse | undefined): SelectOutcome {
  if (!response) return { kind: 'failed' }
  if (response.status === 'requires_credentials' && response.login_hint) {
    return { kind: 'login', loginHint: response.login_hint }
  }
  if (response.url) return { kind: 'redirect', url: response.url }
  const [action] = response.required_actions ?? []
  if (action) return { kind: 'required-action', action }
  return { kind: 'failed' }
}

const UNITS: readonly [Intl.RelativeTimeFormatUnit, number][] = [
  ['day', 86_400],
  ['hour', 3_600],
  ['minute', 60],
]

export function formatLastUsed(
  lastSeenAt: string | null | undefined,
  now: number,
  locale: string
): string | null {
  if (!lastSeenAt) return null
  const seen = Date.parse(lastSeenAt)
  if (Number.isNaN(seen)) return null

  const seconds = Math.max(0, Math.round((now - seen) / 1000))
  const formatter = new Intl.RelativeTimeFormat(locale, { numeric: 'auto' })
  for (const [unit, size] of UNITS) {
    if (seconds >= size) return formatter.format(-Math.floor(seconds / size), unit)
  }
  return formatter.format(0, 'second')
}
