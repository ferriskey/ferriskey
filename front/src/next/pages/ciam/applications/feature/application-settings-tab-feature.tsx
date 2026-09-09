import { useState } from 'react'
import { useNavigate } from 'react-router'
import { toast } from 'sonner'
import { useDeleteClient, useGetClient, useUpdateClient } from '@/api/client.api'
import { useCreateRedirectUri, useDeleteRedirectUri } from '@/api/redirect_uris.api'
import { useCreateWebOrigin, useDeleteWebOrigin, useGetWebOrigins } from '@/api/web_origins.api'
import { DERIVED_ORIGIN_SENTINEL, isWebOriginValue } from '@/lib/web-origin'
import { updateClientSchema } from '@/pages/client/schemas/update-client.schema'
import { Schemas } from '@/api/api.client'
import { CONSOLE_APPLICATIONS_URL } from '../application-routes'
import ApplicationSettingsTab, {
  type ApplicationSettingsDraft,
} from '../ui/application-settings-tab'

import Client = Schemas.Client

const CALLBACK_PATTERN = /^[a-zA-Z][a-zA-Z0-9+.-]*:\/\/.+/
const CALLBACK_ERROR = 'Enter a full URL such as https://app.acme.com/callback.'
const ORIGIN_ERROR = `Enter an origin such as https://app.acme.com — no path, no wildcard — or ${DERIVED_ORIGIN_SENTINEL} to derive them from the callback URLs`

interface Draft extends ApplicationSettingsDraft {
  key: string
}

const pristineDraft = (application: Client): Draft => ({
  key: application.id,
  name: application.name ?? '',
  enabled: application.enabled ?? false,
  directAccessGrants: application.direct_access_grants_enabled ?? false,
  deviceCodeGrant: application.oauth_device_code_grant_enabled ?? false,
  requirePkce: application.require_pkce ?? false,
  accessTokenLifetime: application.access_token_lifetime ?? null,
  refreshTokenLifetime: application.refresh_token_lifetime ?? null,
  idTokenLifetime: application.id_token_lifetime ?? null,
  temporaryTokenLifetime: application.temporary_token_lifetime ?? null,
})

export interface ApplicationSettingsTabFeatureProps {
  application: Client
  realm: string
}

export default function ApplicationSettingsTabFeature({
  application,
  realm,
}: ApplicationSettingsTabFeatureProps) {
  const navigate = useNavigate()

  const { refetch } = useGetClient({ realm, clientId: application.id })
  const { mutate: updateClient } = useUpdateClient()
  const { mutateAsync: deleteClient } = useDeleteClient()

  const { mutateAsync: createRedirectUri } = useCreateRedirectUri()
  const { mutateAsync: deleteRedirectUri } = useDeleteRedirectUri()

  const { data: webOrigins = [] } = useGetWebOrigins({ realmName: realm, clientId: application.id })
  const { mutateAsync: createWebOrigin } = useCreateWebOrigin()
  const { mutateAsync: deleteWebOrigin } = useDeleteWebOrigin()

  const pristine = pristineDraft(application)
  const [draft, setDraft] = useState<Draft>(pristine)
  const [callbackError, setCallbackError] = useState<string>()
  const [originError, setOriginError] = useState<string>()

  if (draft.key !== application.id) setDraft(pristine)

  const current = draft.key === application.id ? draft : pristine

  const parsed = updateClientSchema.safeParse({
    ...current,
    clientId: application.client_id,
  })
  const errors = parsed.success
    ? {}
    : { name: parsed.error.issues.find((i) => i.path[0] === 'name')?.message }

  const dirtyKeys = (Object.keys(pristine) as (keyof Draft)[]).filter(
    (key) => key !== 'key' && current[key] !== pristine[key]
  )

  const save = () => {
    if (!parsed.success) return

    updateClient({
      body: {
        client_id: application.client_id,
        name: current.name,
        enabled: current.enabled,
        direct_access_grants_enabled: current.directAccessGrants,
        oauth_device_code_grant_enabled: current.deviceCodeGrant,
        require_pkce: current.requirePkce,
        access_token_lifetime: current.accessTokenLifetime,
        refresh_token_lifetime: current.refreshTokenLifetime,
        id_token_lifetime: current.idTokenLifetime,
        temporary_token_lifetime: current.temporaryTokenLifetime,
      },
      path: { client_id: application.id, realm_name: realm },
    })
  }

  const handleDelete = async () => {
    const deleted = await deleteClient({
      path: { client_id: application.id, realm_name: realm },
    })
      .then(() => true)
      .catch(() => false)

    if (deleted) {
      toast.success('Application deleted')
      navigate(CONSOLE_APPLICATIONS_URL(realm))
    }
  }

  const redirectUris = application.redirect_uris ?? []

  const handleCallbacksChange = async (next: string[]) => {
    const added = next.find((value) => !redirectUris.some((uri) => uri.value === value))
    const removed = redirectUris.find((uri) => !next.includes(uri.value))

    if (added !== undefined) {
      if (!CALLBACK_PATTERN.test(added.trim())) {
        setCallbackError(CALLBACK_ERROR)
        return
      }
      setCallbackError(undefined)
      try {
        await createRedirectUri({
          realmName: realm,
          clientId: application.id,
          payload: { value: added.trim() },
        })
        await refetch()
        toast.success('Callback URL added')
      } catch (error) {
        toast.error(error instanceof Error ? error.message : 'Failed to add the callback URL')
      }
      return
    }

    if (removed) {
      try {
        await deleteRedirectUri({
          realmName: realm,
          clientId: application.id,
          redirectUriId: removed.id,
        })
        await refetch()
        toast.success('Callback URL removed')
      } catch (error) {
        toast.error(error instanceof Error ? error.message : 'Failed to remove the callback URL')
      }
    }
  }

  const handleOriginsChange = async (next: string[]) => {
    const added = next.find((value) => !webOrigins.some((origin) => origin.value === value))
    const removed = webOrigins.find((origin) => !next.includes(origin.value))

    if (added !== undefined) {
      if (!isWebOriginValue(added)) {
        setOriginError(ORIGIN_ERROR)
        return
      }
      setOriginError(undefined)
      try {
        await createWebOrigin({
          realmName: realm,
          clientId: application.id,
          payload: { value: added.trim() },
        })
        toast.success('Web origin added')
      } catch (error) {
        toast.error(error instanceof Error ? error.message : 'Failed to add the web origin')
      }
      return
    }

    if (removed) {
      try {
        await deleteWebOrigin({
          realmName: realm,
          clientId: application.id,
          webOriginId: removed.id,
        })
        toast.success('Web origin removed')
      } catch (error) {
        toast.error(error instanceof Error ? error.message : 'Failed to remove the web origin')
      }
    }
  }

  return (
    <ApplicationSettingsTab
      application={application}
      draft={current}
      errors={errors}
      dirtyCount={dirtyKeys.length}
      callbacks={redirectUris.map((uri) => uri.value)}
      callbackError={callbackError}
      origins={webOrigins.map((origin) => origin.value)}
      originError={originError}
      onDraftChange={(patch) => setDraft((d) => ({ ...d, ...patch }))}
      onCallbacksChange={(next) => void handleCallbacksChange(next)}
      onOriginsChange={(next) => void handleOriginsChange(next)}
      onDiscard={() => setDraft(pristine)}
      onSave={save}
      onDelete={() => void handleDelete()}
    />
  )
}
