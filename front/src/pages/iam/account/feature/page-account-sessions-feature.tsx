import { useParams } from 'react-router'
import { useTranslation } from 'react-i18next'
import { toast } from 'sonner'
import { useGetOwnProfile } from '@/api/user.api.ts'
import {
  useGetOwnSessions,
  useRevokeOtherOwnSessions,
  useRevokeOwnSession,
  type OwnSessionsQuery,
} from '@/api/account-security.api.ts'
import { usePagedListing } from '@/components/kit'
import type { RouterParams } from '@/routes/router'
import PageAccountSessions from '../ui/page-account-sessions'
import { apiErrorMessage } from '@/lib/api-error'

export default function PageAccountSessionsFeature() {
  const { realm_name } = useParams<RouterParams>()
  const { t } = useTranslation('account')
  const { data: profileResponse, isLoading: isProfileLoading } = useGetOwnProfile({ realm: realm_name })

  const listing = usePagedListing([])
  const { data: sessionsResponse, isLoading: isSessionsLoading } = useGetOwnSessions({
    realm: realm_name,
    query: listing.apiQuery as OwnSessionsQuery,
    keepPrevious: true,
  })
  const { mutate: revokeSession } = useRevokeOwnSession()
  const { mutate: revokeOtherSessions } = useRevokeOtherOwnSessions()

  function handleRevoke(sessionId: string) {
    if (!realm_name) return
    revokeSession(
      { path: { realm_name, session_id: sessionId } },
      {
        onSuccess: () => toast.success(t('sessions.toast.revoked')),
        onError: (error) => toast.error(apiErrorMessage(error)),
      }
    )
  }

  function handleRevokeOthers() {
    if (!realm_name) return
    revokeOtherSessions(
      { path: { realm_name } },
      {
        onSuccess: () => toast.success(t('sessions.toast.others_revoked')),
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
      onRevoke={handleRevoke}
      onRevokeOthers={handleRevokeOthers}
    />
  )
}
