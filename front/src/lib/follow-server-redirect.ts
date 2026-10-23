const REFUSED_SCHEMES = new Set(['javascript', 'data', 'vbscript', 'blob', 'file', 'about'])

export function isNavigableRedirect(target: unknown): target is string {
  if (typeof target !== 'string') return false
  const trimmed = target.trim()
  if (trimmed === '') return false

  const compact = [...trimmed].filter((char) => char.charCodeAt(0) > 0x20).join('')
  const scheme = /^([a-z][a-z0-9+.-]*):/i.exec(compact)?.[1]
  if (!scheme) return true

  return !REFUSED_SCHEMES.has(scheme.toLowerCase())
}

export function followServerRedirect(target: unknown): void {
  if (!isNavigableRedirect(target)) {
    console.error('Refused to follow a redirect target that is not a navigable URL')
    return
  }

  window.location.href = target
}
