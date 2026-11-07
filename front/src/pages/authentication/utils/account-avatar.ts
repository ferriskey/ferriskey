const AVATAR_COLORS = [
  'border-fk-primary-border bg-fk-primary-soft text-fk-primary-text',
  'border-fk-info-border bg-fk-info-soft text-fk-info',
  'border-fk-success-border bg-fk-success-soft text-fk-success',
  'border-fk-violet-border bg-fk-violet-soft text-fk-violet',
] as const

export function avatarInitial(name: string): string {
  const first = Array.from(name.trim())[0]
  return first ? first.toUpperCase() : '?'
}

export function avatarColorClass(displayName: string, maskedEmail: string | null): string {
  const seed = displayName + (maskedEmail ?? '')
  let hash = 0
  for (const char of seed) {
    hash = (hash * 31 + (char.codePointAt(0) ?? 0)) >>> 0
  }
  hash ^= hash >>> 16
  hash = Math.imul(hash, 0x85ebca6b)
  hash ^= hash >>> 13
  hash = Math.imul(hash, 0xc2b2ae35)
  hash ^= hash >>> 16
  return AVATAR_COLORS[(hash >>> 0) % AVATAR_COLORS.length]
}
