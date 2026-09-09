import { useState } from 'react'
import { useNavigate, useParams } from 'react-router'
import { toast } from 'sonner'
import {
  useDeleteIdentityProvider,
  useIdentityProvider,
  useUpdateIdentityProvider,
} from '@/api/identity-providers.api'
import PageProviderDetail from '../ui/page-provider-detail'
import { useCrumbLabel } from '@/next/shell/crumb-store'
import { useIdentityProvidersBase } from '@/next/shared/use-section-base'

interface Draft {
  key: string
  displayName: string
  enabled: boolean
}

const EMPTY_DRAFT: Draft = { key: '', displayName: '', enabled: true }

export default function PageProviderDetailFeature() {
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
      }
    : EMPTY_DRAFT

  if (provider && draft.key !== providerKey) setDraft(pristine)

  const { displayName, enabled } = draft.key === providerKey ? draft : pristine

  const dirtyCount = provider
    ? (displayName !== (provider.display_name ?? '') ? 1 : 0) +
      (enabled !== provider.enabled ? 1 : 0)
    : 0

  const listUrl = identityProvidersBase

  const callbackUrl = `${window.apiUrl}/realms/${realm}/broker/${providerAlias}/endpoint`

  const handleSave = () => {
    if (!provider) return

    updateProvider(
      {
        path: { realm_name: realm, alias: providerAlias },
        body: { display_name: displayName, enabled },
      },
      { onSuccess: () => toast.success('Provider updated successfully') }
    )
  }

  const handleDelete = () => {
    if (!provider) return

    deleteProvider(
      { path: { realm_name: realm, alias: providerAlias } },
      {
        onSuccess: () => {
          toast.success('Provider deleted successfully')
          navigate(listUrl)
        },
        onError: () => toast.error('Failed to delete provider'),
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
      callbackUrl={callbackUrl}
      dirtyCount={dirtyCount}
      onDisplayNameChange={(value) => setDraft((d) => ({ ...d, displayName: value }))}
      onEnabledChange={(value) => setDraft((d) => ({ ...d, enabled: value }))}
      onBack={() => navigate(listUrl)}
      onDiscard={() => setDraft(pristine)}
      onSave={handleSave}
      onDelete={handleDelete}
    />
  )
}
