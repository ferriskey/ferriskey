export function readLoginHint(search: string): string {
  return new URLSearchParams(search).get('login_hint')?.trim() ?? ''
}
