const DEFAULT_REALM = 'master'

export function validateCallbackParams({
  code,
  returnedState,
  expectedState,
}: {
  code: string | null
  returnedState: string | null
  expectedState: string | null
}) {
  if (!code) {
    return 'Missing authorization code. Please try again.'
  }

  if (!returnedState || !expectedState || returnedState !== expectedState) {
    return 'Invalid login state. Please try signing in again.'
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

export function getTokenExchangeErrorMessage(error: unknown) {
  if (error instanceof Error && error.message.trim().length > 0) {
    return error.message
  }

  return 'Unable to complete sign in. Please try again.'
}
