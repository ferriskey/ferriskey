export type LoginSessionAction = 'discard-stale-session' | 'enter-console' | 'show-login-form'

export type LoginSessionInput = {
  isAuthenticated: boolean
  isAuthInitiated: boolean
  sessionExpired: boolean
}

export function resolveLoginSession({
  isAuthenticated,
  isAuthInitiated,
  sessionExpired,
}: LoginSessionInput): LoginSessionAction {
  if (sessionExpired) {
    return 'discard-stale-session'
  }

  if (!isAuthenticated) {
    return 'show-login-form'
  }

  return isAuthInitiated ? 'discard-stale-session' : 'enter-console'
}
