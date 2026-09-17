const DEFAULT_REALM = 'master'

export type CallbackParamsError = 'missing_code' | 'invalid_state'

export function validateCallbackParams({
  code,
  returnedState,
  expectedState,
}: {
  code: string | null
  returnedState: string | null
  expectedState: string | null
}): CallbackParamsError | null {
  if (!code) {
    return 'missing_code'
  }

  if (!returnedState || !expectedState || returnedState !== expectedState) {
    return 'invalid_state'
  }

  return null
}

export function buildLoginErrorRedirect(realmName: string | undefined, errorMessage: string) {
  const realm = realmName ?? DEFAULT_REALM
  const params = new URLSearchParams({
    login_error: errorMessage,
  })

  return `/realms/${realm}/authentication/login?${params.toString()}`
}
