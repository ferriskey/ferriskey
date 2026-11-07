import { useCallback, useEffect, useMemo, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useLocation, useNavigate, useParams } from 'react-router'
import { useQueryClient } from '@tanstack/react-query'
import {
  accountsQueryKey,
  useGetAccounts,
  useRemoveAccount,
  useSelectAccount,
} from '@/api/account-chooser.api'
import { followServerRedirect } from '@/lib/follow-server-redirect'
import { AUTH_NAMESPACE } from '../constants'
import PageAccountChooser from '../ui/page-account-chooser'
import {
  buildLoginPath,
  parseMaxAge,
  resolveSelectOutcome,
} from '../utils/account-chooser'

export default function PageAccountChooserFeature() {
  const navigate = useNavigate()
  const queryClient = useQueryClient()
  const { realm_name } = useParams()
  const { search } = useLocation()
  const { t } = useTranslation(AUTH_NAMESPACE)

  const realm = realm_name ?? 'master'
  const clientId = useMemo(() => new URLSearchParams(search).get('client_id') ?? '', [search])
  const maxAge = useMemo(() => parseMaxAge(search), [search])

  const { data, isPending, isError } = useGetAccounts({ realm, clientId, maxAge })
  const { mutateAsync: selectAccount } = useSelectAccount()
  const { mutateAsync: removeAccount } = useRemoveAccount()

  const [busySessionId, setBusySessionId] = useState<string | null>(null)
  const [errorMessage, setErrorMessage] = useState<string | null>(null)

  const entries = data?.data
  const loginPath = useCallback(
    (loginHint?: string) => buildLoginPath(realm, search, loginHint),
    [realm, search]
  )

  const missingClient = clientId === ''
  const shouldLeave = missingClient || isError || (entries !== undefined && entries.length === 0)

  useEffect(() => {
    if (shouldLeave) navigate(loginPath(), { replace: true })
  }, [shouldLeave, navigate, loginPath])

  const onContinue = useCallback(
    async (sessionId: string) => {
      setBusySessionId(sessionId)
      setErrorMessage(null)
      try {
        const response = await selectAccount({
          path: { realm_name: realm },
          query: { max_age: maxAge },
          body: { session_id: sessionId },
        })
        const outcome = resolveSelectOutcome(response)

        if (outcome.kind === 'redirect') {
          followServerRedirect(outcome.url)
          return
        }
        if (outcome.kind === 'login') {
          navigate(loginPath(outcome.loginHint))
          return
        }
        if (outcome.kind === 'required-action') {
          navigate(
            `/realms/${realm}/authentication/required-action?execution=${outcome.action.toUpperCase()}`
          )
          return
        }
        setErrorMessage(t('account_chooser.error.select_failed'))
      } catch {
        setErrorMessage(t('account_chooser.error.select_failed'))
      }
      setBusySessionId(null)
    },
    [selectAccount, realm, maxAge, navigate, loginPath, t]
  )

  const onRemove = useCallback(
    async (sessionId: string) => {
      setBusySessionId(sessionId)
      setErrorMessage(null)
      try {
        await removeAccount({ path: { realm_name: realm, session_id: sessionId } })
        await queryClient.invalidateQueries({ queryKey: accountsQueryKey(realm, clientId, maxAge) })
      } catch {
        setErrorMessage(t('account_chooser.error.remove_failed'))
      }
      setBusySessionId(null)
    },
    [removeAccount, queryClient, realm, clientId, maxAge, t]
  )

  const accounts = useMemo(
    () =>
      (entries ?? []).map((entry) => ({
        sessionId: entry.session_id,
        displayName: entry.display_name,
        maskedEmail: entry.masked_email ?? null,
        requiresReauthentication: entry.requires_reauthentication,
      })),
    [entries]
  )

  return (
    <PageAccountChooser
      isLoading={isPending || shouldLeave}
      accounts={accounts}
      busySessionId={busySessionId}
      errorMessage={errorMessage}
      onContinue={onContinue}
      onRemove={onRemove}
      onUseAnother={() => navigate(loginPath())}
    />
  )
}
