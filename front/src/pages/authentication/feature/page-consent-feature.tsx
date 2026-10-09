import { useMemo, useState } from 'react'
import { useLocation, useParams } from 'react-router'
import { useTranslation } from 'react-i18next'
import { toConsentScopeList, useGetConsentRequest, useSubmitConsent } from '@/api/consent.api'
import { apiErrorMessage } from '@/lib/api-error'
import {
  approvedScopesForAllow,
  approvedScopesForDeny,
  defaultApprovedOptionalScopes,
  toggleOptionalScope,
} from '../utils/consent-decision'
import PageConsent from '../ui/page-consent'
import { AUTH_NAMESPACE } from '../constants'

export default function PageConsentFeature() {
  const { realm_name } = useParams()
  const realm = realm_name ?? 'master'
  const location = useLocation()
  const { t } = useTranslation(AUTH_NAMESPACE)

  const consentToken = useMemo(
    () => new URLSearchParams(location.search).get('consent_token') ?? '',
    [location.search],
  )

  const {
    data,
    isLoading: isLoadingRequest,
    error: requestError,
  } = useGetConsentRequest({
    realm,
    consentToken,
    enabled: consentToken.length > 0,
  })

  const defaultScopes = useMemo(
    () => toConsentScopeList(data?.granted_by_default ?? []),
    [data],
  )
  const optionalScopes = useMemo(
    () => toConsentScopeList(data?.awaiting_decision ?? []),
    [data],
  )
  const [approvedOverride, setApprovedOverride] = useState<Set<string> | null>(null)
  const approvedOptional = approvedOverride ?? defaultApprovedOptionalScopes(optionalScopes)

  const { mutate: submitConsent, isPending: isSubmittingConsent } = useSubmitConsent()
  const [pendingAction, setPendingAction] = useState<'allow' | 'deny' | null>(null)
  const [errorMessage, setErrorMessage] = useState<string | null>(null)

  const submit = (approvedScopes: string[], action: 'allow' | 'deny') => {
    if (!consentToken) {
      setErrorMessage(t('consent.token_missing'))
      return
    }
    setErrorMessage(null)
    setPendingAction(action)
    submitConsent(
      {
        path: { realm_name: realm },
        body: { consent_token: consentToken, approved_scopes: approvedScopes },
      },
      {
        onSuccess: (response) => {
          window.location.href = response.redirect_url
        },
        onError: (error) => {
          setPendingAction(null)
          setErrorMessage(apiErrorMessage(error, t('consent.failed')))
        },
      },
    )
  }

  const onToggleScope = (name: string) =>
    setApprovedOverride((current) => toggleOptionalScope(current ?? approvedOptional, name))
  const onAllow = () => submit(approvedScopesForAllow(optionalScopes, approvedOptional), 'allow')
  const onDeny = () => submit(approvedScopesForDeny(), 'deny')

  return (
    <PageConsent
      clientName={data?.client_name ?? ''}
      clientUriHost={data?.client_uri_host ?? null}
      defaultScopes={defaultScopes}
      optionalScopes={optionalScopes}
      approvedOptional={approvedOptional}
      onToggleScope={onToggleScope}
      onAllow={onAllow}
      onDeny={onDeny}
      isLoading={!consentToken ? false : isLoadingRequest || !data}
      isAllowing={isSubmittingConsent && pendingAction === 'allow'}
      isDenying={isSubmittingConsent && pendingAction === 'deny'}
      errorMessage={
        errorMessage ??
        (!consentToken
          ? t('consent.token_missing')
          : requestError
            ? apiErrorMessage(requestError, t('consent.failed'))
            : null)
      }
    />
  )
}
