import { useMemo } from 'react'
import { useTranslation } from 'react-i18next'
import { useNavigate, useParams } from 'react-router'
import { toast } from 'sonner'
import {
  useDeleteIdentityProvider,
  useGetIdentityProviders,
} from '@/api/identity-providers.api'
import { useConfirmDeleteAlert } from '@/hooks/use-confirm-delete-alert'
import { useCreatePicker } from '@/components/kit'
import { RouterParams } from '@/routes/router'
import { Schemas } from '@/api/api.client'
import { getTemplateById } from '@/constants/identity-provider-templates'
import { providerName, type ProviderProtocol } from '../provider-status'
import PageProvidersOverview from '../ui/page-providers-overview'

import IdentityProvider = Schemas.IdentityProviderResponse
import { useIdentityProvidersBase } from '@/hooks/use-section-base'

export default function PageProvidersOverviewFeature() {
  const { t } = useTranslation('identity-provider')
  const identityProvidersBase = useIdentityProvidersBase()
  const { realm_name } = useParams<RouterParams>()
  const navigate = useNavigate()
  const realm = realm_name ?? 'master'

  const { data: providersResponse, isLoading } = useGetIdentityProviders({ realm })
  const { mutate: deleteProvider } = useDeleteIdentityProvider()
  const { confirm, ask, close } = useConfirmDeleteAlert()
  const { open, setOpen } = useCreatePicker()

  const providers = useMemo(
    () => providersResponse?.data ?? [],
    [providersResponse]
  )

  const listUrl = identityProvidersBase

  const handleDelete = (provider: IdentityProvider) => {
    const name = providerName(provider)

    ask({
      title: t('list.delete.title'),
      description: t('list.delete.description', { name }),
      onConfirm: () => {
        deleteProvider(
          { path: { realm_name: realm, alias: provider.alias } },
          {
            onSuccess: () => toast.success(t('list.delete.success', { name })),
            onError: () => toast.error(t('list.delete.error', { name })),
          }
        )
        close()
      },
    })
  }

  const handleQuickCreate = (templateId: string) => {
    const template = getTemplateById(templateId)
    if (!template) return

    navigate(
      `${listUrl}/create?protocol=${template.provider_type}&provider=${encodeURIComponent(templateId)}`
    )
  }

  return (
    <PageProvidersOverview
      providers={providers}
      isLoading={isLoading}
      pickerOpen={open}
      confirm={confirm}
      providerHref={(provider) => `${listUrl}/${provider.alias}`}
      createUrl={(protocol: ProviderProtocol) => `${listUrl}/create?protocol=${protocol}`}
      onPickerOpenChange={setOpen}
      onQuickCreate={handleQuickCreate}
      onDelete={handleDelete}
      onConfirmClose={close}
    />
  )
}
