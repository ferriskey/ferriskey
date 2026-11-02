import { useTranslation } from 'react-i18next'
import { Navigate, useNavigate, useParams, useSearchParams } from 'react-router'
import { toast } from 'sonner'
import { useDeleteIdentityProvider } from '@/api/identity-providers.api'
import { useConfirmDeleteAlert } from '@/hooks/use-confirm-delete-alert'
import type { RouterParams } from '@/routes/router'
import type { Schemas } from '@/api/api.client'
import { getTemplateById } from '@/constants/identity-provider-templates'
import { providerName } from '../provider-status'
import PageProvidersOverview from '../ui/page-providers-overview'
import { useProvidersOverview } from './use-providers-overview'

type IdentityProvider = Schemas.IdentityProviderResponse
import { useIdentityProvidersBase } from '@/hooks/use-section-base'

export default function PageProvidersOverviewFeature() {
  const { t } = useTranslation('identity-provider')
  const identityProvidersBase = useIdentityProvidersBase()
  const { realm_name } = useParams<RouterParams>()
  const navigate = useNavigate()
  const realm = realm_name ?? 'master'

  const overview = useProvidersOverview(realm)
  const { mutate: deleteProvider } = useDeleteIdentityProvider()
  const { confirm, ask, close } = useConfirmDeleteAlert()
  const [searchParams] = useSearchParams()

  const listUrl = identityProvidersBase

  const createUrl = `${listUrl}/create`

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

    navigate(`${createUrl}?provider=${encodeURIComponent(templateId)}`)
  }

  if (searchParams.get('create') === '1') return <Navigate to={createUrl} replace />

  return (
    <PageProvidersOverview
      {...overview}
      confirm={confirm}
      providerHref={(provider) => `${listUrl}/${provider.alias}`}
      onCreate={() => navigate(createUrl)}
      onQuickCreate={handleQuickCreate}
      onDelete={handleDelete}
      onConfirmClose={close}
    />
  )
}
