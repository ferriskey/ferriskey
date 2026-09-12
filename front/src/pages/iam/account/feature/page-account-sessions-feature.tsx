import { useMemo } from 'react'
import { useParams } from 'react-router'
import { toast } from 'sonner'
import { useGetOwnProfile, useGetUserSessions, useRevokeUserSession } from '@/api/user.api.ts'
import { authStore } from '@/store/auth.store'
import { RouterParams } from '@/routes/router'
import PageAccountSessions from '../ui/page-account-sessions'

function decodeSid(token: string | null): string | null {
  if (!token) return null
  try {
    const payload = token.split('.')[1]
    if (!payload) return null
    const normalized = payload.replace(/-/g, '+').replace(/_/g, '/')
    const padded = normalized.padEnd(Math.ceil(normalized.length / 4) * 4, '=')
    const claims = JSON.parse(atob(padded)) as Record<string, unknown>
    return typeof claims.sid === 'string' ? claims.sid : null
  } catch {
    return null
  }
}

export default function PageAccountSessionsFeature() {
  const { realm_name } = useParams<RouterParams>()
  const { accessToken } = authStore()
  const { data: profileResponse, isLoading: isProfileLoading } = useGetOwnProfile({ realm: realm_name })
  const userId = profileResponse?.data.id

  const { data: sessionsResponse, isLoading: isSessionsLoading } = useGetUserSessions({
    realm: realm_name,
    userId,
  })
  const { mutate: revokeSession } = useRevokeUserSession()

  const currentSessionId = useMemo(() => decodeSid(accessToken), [accessToken])

  function handleRevoke(sessionId: string) {
    if (!realm_name || !userId) return
    revokeSession(
      {
        path: { realm_name, user_id: userId, session_id: sessionId },
      },
      {
        onSuccess: () => toast.success('Session revoked'),
        onError: (error) => toast.error(error.message),
      }
    )
  }

  return (
    <PageAccountSessions
      profile={profileResponse?.data}
      isLoading={isProfileLoading || isSessionsLoading}
      sessions={sessionsResponse?.data ?? []}
      currentSessionId={currentSessionId}
      onRevoke={handleRevoke}
    />
  )
}
