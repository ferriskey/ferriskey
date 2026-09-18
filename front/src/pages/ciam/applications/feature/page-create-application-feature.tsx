import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Navigate, useNavigate, useParams, useSearchParams } from 'react-router'
import { toast } from 'sonner'
import { useCreateClient } from '@/api/client.api'
import { useCreateRedirectUri } from '@/api/redirect_uris.api'
import { useCreateWebOrigin } from '@/api/web_origins.api'
import { RouterParams } from '@/routes/router'
import { createClientSchema } from '@/pages/iam/client/schemas/create-client.schema'
import { DERIVED_ORIGIN_SENTINEL, isWebOriginValue } from '@/lib/web-origin'
import {
  CONSOLE_APPLICATIONS_URL,
  CONSOLE_APPLICATION_PICKER_URL,
} from '../application-routes'
import {
  applicationFieldsShape,
  createPayloadFor,
  isApplicationType,
} from '../application-types'
import PageCreateApplication, {
  type CreateApplicationErrors,
} from '../ui/page-create-application'
import { apiErrorMessage } from '@/lib/api-error'

const CONSOLE_NAMESPACES = ['console', 'client'] as const

const DEFAULT_APPLICATION_TYPE = 'spa'

const CLIENT_ID_PATTERN = /^[a-z0-9-_]+$/
const CALLBACK_PATTERN = /^[a-zA-Z][a-zA-Z0-9+.-]*:\/\/.+/

const slugify = (value: string) =>
  value
    .toLowerCase()
    .normalize('NFD')
    .replace(/[\u0300-\u036f]/g, '')
    .replace(/[^a-z0-9-_]+/g, '-')
    .replace(/^-+|-+$/g, '')

export default function PageCreateApplicationFeature() {
  const { t } = useTranslation(CONSOLE_NAMESPACES)
  const { realm_name } = useParams<RouterParams>()
  const [params] = useSearchParams()
  const navigate = useNavigate()
  const realm = realm_name ?? 'master'

  const { mutateAsync: createClient } = useCreateClient()
  const { mutateAsync: createRedirectUri } = useCreateRedirectUri()
  const { mutateAsync: createWebOrigin } = useCreateWebOrigin()

  const [name, setName] = useState('')
  const [clientIdOverride, setClientIdOverride] = useState<string | null>(null)
  const [callbacks, setCallbacks] = useState<string[]>([])
  const [origins, setOrigins] = useState<string[]>([])
  const [callbackError, setCallbackError] = useState<string>()
  const [originError, setOriginError] = useState<string>()
  const [submitting, setSubmitting] = useState(false)

  const listUrl = CONSOLE_APPLICATIONS_URL(realm)
  const pickerUrl = CONSOLE_APPLICATION_PICKER_URL(realm)
  const raw = params.get('type')

  const clientId = clientIdOverride ?? slugify(name)
  const fields = applicationFieldsShape(
    isApplicationType(raw) ? raw : DEFAULT_APPLICATION_TYPE
  )

  const identity = createClientSchema.safeParse({ clientId, name })
  const clientIdFormatError =
    clientId.length > 0 && !CLIENT_ID_PATTERN.test(clientId)
      ? t('applications.create.validation.client_id_format')
      : undefined
  const callbackRequirementError =
    fields.callbackRequired && callbacks.length === 0
      ? t('applications.create.validation.callback_required')
      : undefined

  const errors: CreateApplicationErrors = {
    name: identity.success
      ? undefined
      : identity.error.issues.find((i) => i.path[0] === 'name')?.message,
    clientId:
      clientIdFormatError ??
      (identity.success
        ? undefined
        : identity.error.issues.find((i) => i.path[0] === 'clientId')?.message),
    callbacks: callbackError ?? callbackRequirementError,
    origins: originError,
  }

  const canSubmit =
    identity.success &&
    !clientIdFormatError &&
    !callbackRequirementError &&
    !submitting

  const handleCallbacksChange = (next: string[]) => {
    const added = next.find((value) => !callbacks.includes(value))

    if (added !== undefined && !CALLBACK_PATTERN.test(added.trim())) {
      setCallbackError(t('applications.create.validation.callback_format'))
      return
    }

    setCallbackError(undefined)
    setCallbacks(next)
  }

  const handleOriginsChange = (next: string[]) => {
    const added = next.find((value) => !origins.includes(value))

    if (added !== undefined && !isWebOriginValue(added)) {
      setOriginError(
        t('applications.create.validation.origin_format', {
          sentinel: DERIVED_ORIGIN_SENTINEL,
        })
      )
      return
    }

    setOriginError(undefined)
    setOrigins(next)
  }

  const handleSubmit = async () => {
    if (!canSubmit || !isApplicationType(raw)) return

    setSubmitting(true)
    try {
      const created = await createClient({
        path: { realm_name: realm },
        body: createPayloadFor(raw, name.trim(), clientId.trim()),
      })

      for (const value of callbacks) {
        try {
          await createRedirectUri({
            realmName: realm,
            clientId: created.id,
            payload: { value },
          })
        } catch {
          toast.error(t('applications.create.toast.callback_failed', { value }))
        }
      }

      for (const value of origins) {
        try {
          await createWebOrigin({
            realmName: realm,
            clientId: created.id,
            payload: { value },
          })
        } catch {
          toast.error(t('applications.create.toast.origin_failed', { value }))
        }
      }

      toast.success(t('applications.create.toast.created'))
      navigate(listUrl)
    } catch (error) {
      toast.error(apiErrorMessage(error, t('applications.create.toast.create_failed')))
    } finally {
      setSubmitting(false)
    }
  }

  if (!isApplicationType(raw)) return <Navigate to={pickerUrl} replace />

  return (
    <PageCreateApplication
      type={raw}
      listUrl={listUrl}
      pickerUrl={pickerUrl}
      name={name}
      clientId={clientId}
      callbacks={callbacks}
      origins={origins}
      errors={errors}
      canSubmit={canSubmit}
      onNameChange={setName}
      onClientIdChange={setClientIdOverride}
      onCallbacksChange={handleCallbacksChange}
      onOriginsChange={handleOriginsChange}
      onCancel={() => navigate(listUrl)}
      onSubmit={() => void handleSubmit()}
    />
  )
}
