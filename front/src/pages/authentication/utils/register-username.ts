export function resolveRegisterUsername(username: string, email: string): string {
  return username.trim() || email.trim().toLowerCase()
}
