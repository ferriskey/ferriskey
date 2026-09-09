import { useMemo } from 'react'
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
import { useIdentityProvidersBase } from '@/next/shared/use-section-base'

export default function PageProvidersOverviewFeature() {
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
      title: 'Delete provider?',
      description: `Are you sure you want to delete "${name}"?`,
      onConfirm: () => {
        deleteProvider(
          { path: { realm_name: realm, alias: provider.alias } },
          {
            onSuccess: () => toast.success(`Provider "${name}" deleted`),
            onError: () => toast.error(`Failed to delete "${name}"`),
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
