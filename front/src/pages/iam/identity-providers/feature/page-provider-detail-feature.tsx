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
  storedProviderDraft,
  validateProviderDraft,
  type ProviderDraft,
} from '../provider-update-body'

interface Draft extends ProviderDraft {
  key: string
}

const EMPTY_DRAFT: Draft = {
  key: '',
  displayName: '',
  enabled: true,
  clientId: '',
  clientSecret: '',
  authorizationUrl: '',
  tokenUrl: '',
  userinfoUrl: '',
  scopes: [],
  usePkce: false,
}

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
    ? { key: providerKey, ...storedProviderDraft(provider) }
    : EMPTY_DRAFT

  if (provider && draft.key !== providerKey) setDraft(pristine)

  const values: Draft = draft.key === providerKey ? draft : pristine

  const errors = validateProviderDraft(values)
  const dirtyCount = provider ? countProviderChanges(values, pristine) : 0
  const canSave = dirtyCount > 0 && Object.keys(errors).length === 0

  const listUrl = identityProvidersBase

  const callbackUrl = `${window.apiUrl}/realms/${realm}/broker/${providerAlias}/endpoint`

  const handleSave = () => {
    if (!provider || !canSave) return

    updateProvider(
      {
        path: { realm_name: realm, alias: providerAlias },
        body: buildProviderUpdateBody(values, pristine),
      },
      {
        onSuccess: () => {
          setDraft({ ...values, clientSecret: '' })
          toast.success(t('detail.update_success'))
        },
        onError: () => toast.error(t('detail.update_error')),
      }
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
      draft={values}
      errors={errors}
      callbackUrl={callbackUrl}
      dirtyCount={dirtyCount}
      canSave={canSave}
      onChange={(patch) => setDraft((current) => ({ ...current, ...patch }))}
      onBack={() => navigate(listUrl)}
      onDiscard={() => setDraft(pristine)}
      onSave={handleSave}
      onDelete={handleDelete}
    />
  )
}
