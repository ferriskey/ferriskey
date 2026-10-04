import { useMemo } from 'react'
import { useParams } from 'react-router'
import { useTranslation } from 'react-i18next'
import { toast } from 'sonner'
import {
  USER_SESSION_FILTER_KEYS,
  useGetOwnProfile,
  useGetUserSessions,
  useRevokeUserSession,
  type UserSessionsQuery,
} from '@/api/user.api.ts'
import { usePagedListing } from '@/components/kit'
import { authStore } from '@/store/auth.store'
import { RouterParams } from '@/routes/router'
import PageAccountSessions from '../ui/page-account-sessions'
import { apiErrorMessage } from '@/lib/api-error'

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
  const { t } = useTranslation('account')
  const { accessToken } = authStore()
  const { data: profileResponse, isLoading: isProfileLoading } = useGetOwnProfile({ realm: realm_name })
  const userId = profileResponse?.data.id

  const listing = usePagedListing(USER_SESSION_FILTER_KEYS)
  const { data: sessionsResponse, isLoading: isSessionsLoading } = useGetUserSessions({
    realm: realm_name,
    userId,
    query: listing.apiQuery as UserSessionsQuery,
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
        onSuccess: () => toast.success(t('sessions.toast.revoked')),
        onError: (error) => toast.error(apiErrorMessage(error)),
      }
    )
  }

  return (
    <PageAccountSessions
      profile={profileResponse?.data}
      isLoading={isProfileLoading}
      isSessionsLoading={isSessionsLoading}
      sessions={sessionsResponse?.data ?? []}
      pagination={sessionsResponse?.metadata}
      listing={listing}
      currentSessionId={currentSessionId}
      onRevoke={handleRevoke}
    />
  )
}
