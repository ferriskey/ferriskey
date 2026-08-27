export const DERIVED_ORIGIN_SENTINEL = '+'

const ORIGIN_SCHEMES = ['http:', 'https:']

export function normalizeWebOrigin(value: string): string | null {
  const candidate = value.trim()

  if (candidate === '' || candidate === '*') {
    return null
  }

  let url: URL
  try {
    url = new URL(candidate)
  } catch {
    return null
  }

  if (!ORIGIN_SCHEMES.includes(url.protocol)) {
    return null
  }

  if (url.username !== '' || url.password !== '') {
    return null
  }

  if (url.pathname !== '' && url.pathname !== '/') {
    return null
  }

  if (url.search !== '' || url.hash !== '') {
    return null
  }

  if (url.hostname.includes('*')) {
    return null
  }

  return url.origin
}

export function isWebOriginValue(value: string): boolean {
  return value.trim() === DERIVED_ORIGIN_SENTINEL || normalizeWebOrigin(value) !== null
}
