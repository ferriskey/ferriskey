import { useDeleteClient, useGetClient, useUpdateClient } from '@/api/client.api'
import { useCreateRedirectUri, useDeleteRedirectUri } from '@/api/redirect_uris.api'
import { RouterParams } from '@/routes/router'
import { APPLICATIONS_URL } from '@/routes/sub-router/applications.router'
import { Skeleton } from '@/components/ui/skeleton'
import { useState } from 'react'
import { useNavigate, useParams } from 'react-router'
import { toast } from 'sonner'
import PageApplicationDetail, { ApplicationSettingsValues } from '../ui/page-application-detail'

export default function PageApplicationDetailFeature() {
  const { realm_name, client_id } = useParams<RouterParams>()
  const navigate = useNavigate()
  const realm = realm_name ?? 'master'

  const { data: clientResponse, isLoading, refetch } = useGetClient({
    realm,
    clientId: client_id ?? '',
  })

  const { mutateAsync: updateClient, isPending: isSaving } = useUpdateClient()
  const { mutateAsync: deleteClient } = useDeleteClient()
  const { mutateAsync: createRedirectUri } = useCreateRedirectUri()
  const { mutateAsync: deleteRedirectUri } = useDeleteRedirectUri()
  const [uriPending, setUriPending] = useState(false)

  const client = clientResponse?.data

  const handleBack = () => navigate(APPLICATIONS_URL(realm))

  const handleSave = async (values: ApplicationSettingsValues) => {
    if (!client) return
    await updateClient({
      body: {
        client_id: client.client_id,
        name: values.name,
        enabled: values.enabled,
        direct_access_grants_enabled: values.directAccessGrantsEnabled,
        oauth_device_code_grant_enabled: values.oauthDeviceCodeGrantEnabled,
        access_token_lifetime: values.accessTokenLifetime,
        refresh_token_lifetime: values.refreshTokenLifetime,
        id_token_lifetime: values.idTokenLifetime,
        temporary_token_lifetime: values.temporaryTokenLifetime,
      },
      path: { client_id: client.id, realm_name: realm },
    })
  }

  const handleDelete = async () => {
    if (!client) return
    try {
      await deleteClient({ path: { client_id: client.id, realm_name: realm } })
      toast.success('Application deleted')
      navigate(APPLICATIONS_URL(realm))
    } catch {
      // error surfaced by the mutation hook
    }
  }

  const handleAddRedirectUri = async (value: string) => {
    if (!client) return
    setUriPending(true)
    try {
      await createRedirectUri({ realmName: realm, clientId: client.id, payload: { value } })
      await refetch()
      toast.success('Redirect URI added')
    } catch (err) {
      toast.error(err instanceof Error ? err.message : 'Failed to add redirect URI')
    } finally {
      setUriPending(false)
    }
  }

  const handleDeleteRedirectUri = async (redirectUriId: string) => {
    if (!client) return
    setUriPending(true)
    try {
      await deleteRedirectUri({ realmName: realm, clientId: client.id, redirectUriId })
      await refetch()
      toast.success('Redirect URI removed')
    } catch (err) {
      toast.error(err instanceof Error ? err.message : 'Failed to remove redirect URI')
    } finally {
      setUriPending(false)
    }
  }

  if (isLoading || !client) {
    return (
      <div className='flex flex-col gap-6 p-8 md:p-12 max-w-3xl'>
        <Skeleton className='h-6 w-48' />
        <Skeleton className='h-40 w-full rounded-md' />
        <Skeleton className='h-40 w-full rounded-md' />
      </div>
    )
  }

  return (
    <PageApplicationDetail
      key={client.id}
      client={client}
      isSaving={isSaving}
      uriPending={uriPending}
      onBack={handleBack}
      onSave={handleSave}
      onDelete={handleDelete}
      onAddRedirectUri={handleAddRedirectUri}
      onDeleteRedirectUri={handleDeleteRedirectUri}
    />
  )
}
