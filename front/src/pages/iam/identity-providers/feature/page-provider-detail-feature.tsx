import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useNavigate, useParams } from 'react-router'
import { toast } from 'sonner'
import {
  useDeleteIdentityProvider,
  useIdentityProvider,
  useUpdateIdentityProvider,
} from '@/api/identity-providers.api'
import PageProviderDetail from '../ui/page-provider-detail'
import { useCrumbLabel } from '@/components/shell/crumb-store'
import { useIdentityProvidersBase } from '@/hooks/use-section-base'
import {
  buildProviderUpdateBody,
  countProviderChanges,
  storedUsePkce,
  type ProviderDraft,
} from '../provider-update-body'

interface Draft extends ProviderDraft {
  key: string
}

const EMPTY_DRAFT: Draft = { key: '', displayName: '', enabled: true, usePkce: false }

export default function PageProviderDetailFeature() {
  const { t } = useTranslation('identity-provider')
  const identityProvidersBase = useIdentityProvidersBase()
  const { realm_name, alias } = useParams<{ realm_name: string; alias: string }>()
  const navigate = useNavigate()
  const realm = realm_name ?? 'master'
  const providerAlias = alias ?? ''

  const { data: provider, isLoading } = useIdentityProvider({
    realm,
    providerId: providerAlias,
  })
  const { mutate: updateProvider } = useUpdateIdentityProvider()
  const { mutate: deleteProvider } = useDeleteIdentityProvider()

  const [draft, setDraft] = useState<Draft>(EMPTY_DRAFT)

  const providerKey = provider?.alias ?? ''
  const pristine: Draft = provider
    ? {
        key: providerKey,
        displayName: provider.display_name ?? '',
        enabled: provider.enabled,
        usePkce: storedUsePkce(provider.config),
      }
    : EMPTY_DRAFT

  if (provider && draft.key !== providerKey) setDraft(pristine)

  const { displayName, enabled, usePkce } = draft.key === providerKey ? draft : pristine

  const dirtyCount = provider
    ? countProviderChanges({ displayName, enabled, usePkce }, pristine)
    : 0

  const listUrl = identityProvidersBase

  const callbackUrl = `${window.apiUrl}/realms/${realm}/broker/${providerAlias}/endpoint`

  const handleSave = () => {
    if (!provider) return

    updateProvider(
      {
        path: { realm_name: realm, alias: providerAlias },
        body: buildProviderUpdateBody({ displayName, enabled, usePkce }, pristine),
      },
      { onSuccess: () => toast.success(t('detail.update_success')) }
    )
  }

  const handleDelete = () => {
    if (!provider) return

    deleteProvider(
      { path: { realm_name: realm, alias: providerAlias } },
      {
        onSuccess: () => {
          toast.success(t('detail.delete_success'))
          navigate(listUrl)
        },
        onError: () => toast.error(t('detail.delete_error')),
      }
    )
  }


  useCrumbLabel(alias, provider?.display_name ?? provider?.alias)

  return (
    <PageProviderDetail
      provider={provider}
      isLoading={isLoading}
      displayName={displayName}
      enabled={enabled}
      usePkce={usePkce}
      callbackUrl={callbackUrl}
      dirtyCount={dirtyCount}
      onDisplayNameChange={(value) => setDraft((d) => ({ ...d, displayName: value }))}
      onEnabledChange={(value) => setDraft((d) => ({ ...d, enabled: value }))}
      onUsePkceChange={(value) => setDraft((d) => ({ ...d, usePkce: value }))}
      onBack={() => navigate(listUrl)}
      onDiscard={() => setDraft(pristine)}
      onSave={handleSave}
      onDelete={handleDelete}
    />
  )
}
