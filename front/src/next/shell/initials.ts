export function getInitials(username?: string): string {
  if (!username) return '??'
  const parts = username.trim().split(/[\s._-]+/)
  if (parts.length >= 2) return (parts[0][0] + parts[1][0]).toUpperCase()
  return username.slice(0, 2).toUpperCase()
}
