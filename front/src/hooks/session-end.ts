export type SessionEndReason = 'logout' | 'expired'

/**
 * An explicit logout ends the realm context, so the next anonymous visit
 * should use the realm in the URL. Expiry and failed refreshes keep it so the
 * user is sent back to the realm that issued the session.
 */
export function sessionRealmAfterSessionEnd(
  sessionRealm: string | null,
  reason: SessionEndReason
): string | null {
  return reason === 'logout' ? null : sessionRealm
}
