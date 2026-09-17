const VERIFY_EMAIL_CONTEXT_KEY = 'ferriskey_verify_email_context'

const CONTEXT_TTL_MS = 30 * 60 * 1000

export interface VerifyEmailContext {
  realm: string
  clientId: string
  timestamp: number
}

export function storeVerifyEmailContext(context: Omit<VerifyEmailContext, 'timestamp'>) {
  sessionStorage.setItem(
    VERIFY_EMAIL_CONTEXT_KEY,
    JSON.stringify({ ...context, timestamp: Date.now() })
  )
}

export function getVerifyEmailContext(): VerifyEmailContext | null {
  const stored = sessionStorage.getItem(VERIFY_EMAIL_CONTEXT_KEY)
  if (!stored) return null

  try {
    const context = JSON.parse(stored) as VerifyEmailContext
    if (Date.now() - context.timestamp > CONTEXT_TTL_MS) {
      clearVerifyEmailContext()
      return null
    }
    return context
  } catch {
    return null
  }
}

export function clearVerifyEmailContext() {
  sessionStorage.removeItem(VERIFY_EMAIL_CONTEXT_KEY)
}
