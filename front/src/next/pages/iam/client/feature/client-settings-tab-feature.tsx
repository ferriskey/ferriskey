import { useState } from 'react'
import { useNavigate } from 'react-router'
import { toast } from 'sonner'
import { useDeleteClient, useGetClient, useUpdateClient } from '@/api/client.api'
import { useCreateRedirectUri, useDeleteRedirectUri } from '@/api/redirect_uris.api'
import { useCreateWebOrigin, useDeleteWebOrigin, useGetWebOrigins } from '@/api/web_origins.api'
import {
  useCreatePostLogoutRedirectUri,
  useDeletePostLogoutRedirectUri,
  useGetPostLogoutRedirectUris,
} from '@/api/post_logout_redirect_uris.api'
import { DERIVED_ORIGIN_SENTINEL, isWebOriginValue } from '@/lib/web-origin'
import { updateClientSchema } from '@/pages/client/schemas/update-client.schema'
import { NEXT_CLIENTS_URL } from '@/next/routes'
import { Schemas } from '@/api/api.client'
import ClientSettingsTab, { type ClientSettingsDraft } from '../ui/client-settings-tab'

import Client = Schemas.Client

const WEB_ORIGIN_ERROR = `Enter an origin such as https://app.example.com — no path, no wildcard — or ${DERIVED_ORIGIN_SENTINEL} to derive them from this client's redirect URIs`

interface Draft extends ClientSettingsDraft {
  key: string
}

const pristineDraft = (client: Client): Draft => ({
  key: client.id,
  name: client.name ?? '',
  clientId: client.client_id ?? '',
  enabled: client.enabled ?? false,
  directAccessGrants: client.direct_access_grants_enabled ?? false,
  deviceCodeGrant: client.oauth_device_code_grant_enabled ?? false,
  requirePkce: client.require_pkce ?? false,
  accessTokenLifetime: client.access_token_lifetime ?? null,
  refreshTokenLifetime: client.refresh_token_lifetime ?? null,
  idTokenLifetime: client.id_token_lifetime ?? null,
  temporaryTokenLifetime: client.temporary_token_lifetime ?? null,
})

export interface ClientSettingsTabFeatureProps {
  client: Client
  realm: string
}

export default function ClientSettingsTabFeature({
  client,
  realm,
}: ClientSettingsTabFeatureProps) {
  const navigate = useNavigate()

  const { refetch } = useGetClient({ realm, clientId: client.id })
  const { mutate: updateClient } = useUpdateClient()
  const { mutateAsync: deleteClient } = useDeleteClient()

  const { mutateAsync: createRedirectUri } = useCreateRedirectUri()
  const { mutateAsync: deleteRedirectUri } = useDeleteRedirectUri()

  const { data: webOrigins = [] } = useGetWebOrigins({ realmName: realm, clientId: client.id })
  const { mutateAsync: createWebOrigin } = useCreateWebOrigin()
  const { mutateAsync: deleteWebOrigin } = useDeleteWebOrigin()

  const { data: postLogoutUris = [] } = useGetPostLogoutRedirectUris({
    realmName: realm,
    clientId: client.id,
  })
  const { mutateAsync: createPostLogoutUri } = useCreatePostLogoutRedirectUri()
  const { mutateAsync: deletePostLogoutUri } = useDeletePostLogoutRedirectUri()

  const pristine = pristineDraft(client)
  const [draft, setDraft] = useState<Draft>(pristine)
  const [redirectUriError, setRedirectUriError] = useState<string>()
  const [webOriginError, setWebOriginError] = useState<string>()
  const [postLogoutRedirectUriError, setPostLogoutRedirectUriError] = useState<string>()

  if (draft.key !== client.id) setDraft(pristine)

  const current = draft.key === client.id ? draft : pristine

  const onDraftChange = (patch: Partial<ClientSettingsDraft>) =>
    setDraft((d) => ({ ...d, ...patch }))

  const parsed = updateClientSchema.safeParse(current)
  const errors = parsed.success
    ? {}
    : {
        clientId: parsed.error.issues.find((i) => i.path[0] === 'clientId')?.message,
        name: parsed.error.issues.find((i) => i.path[0] === 'name')?.message,
      }

  const dirtyKeys = (Object.keys(pristine) as (keyof Draft)[]).filter(
    (key) => key !== 'key' && current[key] !== pristine[key]
  )

  const save = () => {
    if (!parsed.success) return

    updateClient({
      body: {
        client_id: current.clientId,
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
      path: { client_id: client.id, realm_name: realm },
    })
  }

  const handleDelete = async () => {
    const deleted = await deleteClient({ path: { client_id: client.id, realm_name: realm } })
      .then(() => true)
      .catch(() => false)

    if (deleted) navigate(NEXT_CLIENTS_URL(realm))
  }

  const redirectUris = client.redirect_uris ?? []

  const handleRedirectUrisChange = async (next: string[]) => {
    const added = next.find((value) => !redirectUris.some((uri) => uri.value === value))
    const removed = redirectUris.find((uri) => !next.includes(uri.value))

    if (added !== undefined) {
      if (added.trim() === '') {
        setRedirectUriError('Redirect URI is required')
        return
      }
      setRedirectUriError(undefined)
      await createRedirectUri({
        realmName: realm,
        clientId: client.id,
        payload: { value: added },
      })
      await refetch()
      toast.success('Redirect URI added successfully')
      return
    }

    if (removed) {
      await deleteRedirectUri({
        realmName: realm,
        clientId: client.id,
        redirectUriId: removed.id,
      })
      await refetch()
      toast.success('Redirect URI deleted successfully')
    }
  }

  const handleWebOriginsChange = async (next: string[]) => {
    const added = next.find((value) => !webOrigins.some((origin) => origin.value === value))
    const removed = webOrigins.find((origin) => !next.includes(origin.value))

    if (added !== undefined) {
      if (!isWebOriginValue(added)) {
        setWebOriginError(WEB_ORIGIN_ERROR)
        return
      }
      setWebOriginError(undefined)
      try {
        await createWebOrigin({
          realmName: realm,
          clientId: client.id,
          payload: { value: added.trim() },
        })
        toast.success('Web origin added successfully')
      } catch (error) {
        toast.error(error instanceof Error ? error.message : 'Failed to create web origin')
      }
      return
    }

    if (removed) {
      try {
        await deleteWebOrigin({
          realmName: realm,
          clientId: client.id,
          webOriginId: removed.id,
        })
        toast.success('Web origin deleted successfully')
      } catch (error) {
        toast.error(error instanceof Error ? error.message : 'Failed to delete web origin')
      }
    }
  }

  const handlePostLogoutRedirectUrisChange = async (next: string[]) => {
    const added = next.find((value) => !postLogoutUris.some((uri) => uri.value === value))
    const removed = postLogoutUris.find((uri) => !next.includes(uri.value))

    if (added !== undefined) {
      if (added.trim() === '') {
        setPostLogoutRedirectUriError('Post-logout redirect URI is required')
        return
      }
      setPostLogoutRedirectUriError(undefined)
      try {
        await createPostLogoutUri({
          realmName: realm,
          clientId: client.id,
          payload: { value: added },
        })
        toast.success('Post-logout redirect URI added successfully')
      } catch {
        toast.error('Failed to create post-logout redirect URI')
      }
      return
    }

    if (removed) {
      try {
        await deletePostLogoutUri({
          realmName: realm,
          clientId: client.id,
          redirectUriId: removed.id,
        })
        toast.success('Post-logout redirect URI deleted successfully')
      } catch {
        toast.error('Failed to delete post-logout redirect URI')
      }
    }
  }

  return (
    <ClientSettingsTab
      client={client}
      draft={current}
      errors={errors}
      dirtyCount={dirtyKeys.length}
      redirectUris={redirectUris.map((uri) => uri.value)}
      redirectUriError={redirectUriError}
      webOrigins={webOrigins.map((origin) => origin.value)}
      webOriginError={webOriginError}
      postLogoutRedirectUris={postLogoutUris.map((uri) => uri.value)}
      postLogoutRedirectUriError={postLogoutRedirectUriError}
      onDraftChange={onDraftChange}
      onRedirectUrisChange={(next) => void handleRedirectUrisChange(next)}
      onWebOriginsChange={(next) => void handleWebOriginsChange(next)}
      onPostLogoutRedirectUrisChange={(next) => void handlePostLogoutRedirectUrisChange(next)}
      onDiscard={() => setDraft(pristine)}
      onSave={save}
      onDelete={() => void handleDelete()}
    />
  )
}
