export type McpEntryError = 'invalid' | 'duplicate'

const LOOPBACK_HOSTS = new Set(['localhost', '127.0.0.1', '[::1]'])
const BARE_HOST =
  /^(?=.{1,253}$)[a-z0-9]([a-z0-9-]{0,61}[a-z0-9])?(\.[a-z0-9]([a-z0-9-]{0,61}[a-z0-9])?)*(:\d{1,5})?$/i

export function isValidResource(value: string): boolean {
  if (value !== value.trim() || value.includes('#')) return false
  let url: URL
  try {
    url = new URL(value)
  } catch {
    return false
  }
  if (url.protocol === 'https:') return url.hostname.length > 0
  if (url.protocol === 'http:') return LOOPBACK_HOSTS.has(url.hostname)
  return false
}

export function isValidCimdHost(value: string): boolean {
  return BARE_HOST.test(value)
}

export function validateEntry(
  value: string,
  existing: readonly string[],
  isValid: (value: string) => boolean
): McpEntryError | null {
  if (!isValid(value)) return 'invalid'
  if (existing.includes(value)) return 'duplicate'
  return null
}

export function invalidEntries(
  values: readonly string[],
  isValid: (value: string) => boolean
): string[] {
  return values.filter((value) => !isValid(value))
}
