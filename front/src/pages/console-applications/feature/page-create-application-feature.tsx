import { useCreateClient } from '@/api/client.api'
import { useCreateRedirectUri } from '@/api/redirect_uris.api'
import { RouterParams } from '@/routes/router'
import {
  APPLICATIONS_URL,
  APPLICATION_CREATE_URL,
  ApplicationType,
} from '@/routes/sub-router/applications.router'
import { useState } from 'react'
import { useNavigate, useParams } from 'react-router'
import { toast } from 'sonner'
import PageCreateApplication, { CreateApplicationValues } from '../ui/page-create-application'

interface RouteParams extends RouterParams {
  type: ApplicationType
}

const VALID_TYPES: ApplicationType[] = ['native', 'spa', 'web', 'm2m']

function payloadFor(type: ApplicationType, name: string, clientId: string) {
  const base = { name, client_id: clientId, enabled: true, protocol: 'openid-connect' as const }
  switch (type) {
    case 'native':
      return { ...base, client_type: 'public' as const, public_client: false, service_account_enabled: false, direct_access_grants_enabled: false }
    case 'spa':
      return { ...base, client_type: 'public' as const, public_client: true, service_account_enabled: false, direct_access_grants_enabled: false }
    case 'web':
      return { ...base, client_type: 'confidential' as const, public_client: false, service_account_enabled: false, direct_access_grants_enabled: false }
    case 'm2m':
      return { ...base, client_type: 'confidential' as const, public_client: false, service_account_enabled: true, direct_access_grants_enabled: false }
  }
}

export default function PageCreateApplicationFeature() {
  const { realm_name, type } = useParams<RouteParams>()
  const navigate = useNavigate()
  const { mutateAsync: createClient } = useCreateClient()
  const { mutateAsync: createRedirectUri } = useCreateRedirectUri()
  const [submitting, setSubmitting] = useState(false)

  // Guard against an unknown :type segment by sending the user back to step 1.
  if (!realm_name || !type || !VALID_TYPES.includes(type)) {
    navigate(APPLICATION_CREATE_URL(realm_name ?? 'master'), { replace: true })
    return null
  }

  const handleCancel = () => navigate(APPLICATION_CREATE_URL(realm_name))
  const handleBackToTypePicker = () => navigate(APPLICATION_CREATE_URL(realm_name))

  const handleSubmit = async (values: CreateApplicationValues) => {
    setSubmitting(true)
    try {
      const created = await createClient({
        path: { realm_name },
        body: payloadFor(type, values.name, values.clientId),
      })
      const clientId = created.id
      // Best-effort registration of allowed callback URLs. If one fails we
      // surface the error but keep the client (the user can finish in admin).
      const urls = [...values.callbackUrls, ...values.allowedOrigins].filter(Boolean)
      for (const url of urls) {
        try {
          await createRedirectUri({ realmName: realm_name, clientId, payload: { value: url } })
        } catch {
          toast.error(`Could not register URL: ${url}`)
        }
      }
      toast.success('Application created')
      navigate(APPLICATIONS_URL(realm_name))
    } catch (err) {
      toast.error(err instanceof Error ? err.message : 'Failed to create application')
    } finally {
      setSubmitting(false)
    }
  }

  return (
    <PageCreateApplication
      type={type}
      onCancel={handleCancel}
      onBack={handleBackToTypePicker}
      onSubmit={handleSubmit}
      isSubmitting={submitting}
    />
  )
}
