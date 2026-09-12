import { useState } from 'react'
import { Navigate, useNavigate, useParams, useSearchParams } from 'react-router'
import { toast } from 'sonner'
import { useCreateClient } from '@/api/client.api'
import { useCreateRedirectUri } from '@/api/redirect_uris.api'
import { useCreateWebOrigin } from '@/api/web_origins.api'
import { RouterParams } from '@/routes/router'
import { createClientSchema } from '@/pages/iam/client/schemas/create-client.schema'
import { isWebOriginValue } from '@/lib/web-origin'
import {
  CONSOLE_APPLICATIONS_URL,
  CONSOLE_APPLICATION_PICKER_URL,
} from '../application-routes'
import {
  APPLICATION_FIELDS,
  createPayloadFor,
  isApplicationType,
} from '../application-types'
import PageCreateApplication, {
  type CreateApplicationErrors,
} from '../ui/page-create-application'

const CLIENT_ID_PATTERN = /^[a-z0-9-_]+$/
const CLIENT_ID_ERROR = 'Only lowercase letters, numbers, hyphens and underscores.'
const CALLBACK_PATTERN = /^[a-zA-Z][a-zA-Z0-9+.-]*:\/\/.+/
const CALLBACK_ERROR = 'Enter a full URL such as https://app.acme.com/callback.'
const CALLBACK_REQUIRED = 'At least one callback URL is required for this application type.'
const ORIGIN_ERROR =
  'Enter an origin such as https://app.acme.com — no path, no wildcard — or + to derive them from the callback URLs.'

const slugify = (value: string) =>
  value
    .toLowerCase()
    .normalize('NFD')
    .replace(/[\u0300-\u036f]/g, '')
    .replace(/[^a-z0-9-_]+/g, '-')
    .replace(/^-+|-+$/g, '')

export default function PageCreateApplicationFeature() {
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
  const fields = isApplicationType(raw) ? APPLICATION_FIELDS[raw] : APPLICATION_FIELDS.spa

  const identity = createClientSchema.safeParse({ clientId, name })
  const clientIdFormatError =
    clientId.length > 0 && !CLIENT_ID_PATTERN.test(clientId) ? CLIENT_ID_ERROR : undefined
  const callbackRequirementError =
    fields.callbackRequired && callbacks.length === 0 ? CALLBACK_REQUIRED : undefined

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
      setCallbackError(CALLBACK_ERROR)
      return
    }

    setCallbackError(undefined)
    setCallbacks(next)
  }

  const handleOriginsChange = (next: string[]) => {
    const added = next.find((value) => !origins.includes(value))

    if (added !== undefined && !isWebOriginValue(added)) {
      setOriginError(ORIGIN_ERROR)
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
          toast.error(`Could not register callback URL: ${value}`)
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
          toast.error(`Could not register web origin: ${value}`)
        }
      }

      toast.success('Application created')
      navigate(listUrl)
    } catch (error) {
      toast.error(error instanceof Error ? error.message : 'Failed to create application')
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
