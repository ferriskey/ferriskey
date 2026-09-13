export function toConfirmToken(name: string) {
  return name.trim().toLowerCase().replace(/\s+/g, '-')
}
