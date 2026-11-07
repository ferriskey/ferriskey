const AVATAR_COLORS = [
  'bg-teal-600',
  'bg-purple-600',
  'bg-indigo-600',
  'bg-green-700',
  'bg-orange-600',
  'bg-blue-600',
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
