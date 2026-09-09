import { Input } from '@/components/ui/input'
import { DurationInput } from '@/components/ui/duration-input'
import SaveBar from '@/components/kit/save-bar'
import { DangerZone } from '@/components/kit/danger-zone'
import { ChipInput, ChoiceCards, FieldRow, Section, SwitchField } from '@/components/kit'
import { Schemas } from '@/api/api.client'
import {
  lockedAuthenticationChoices,
  stateChoices,
  type ClientState,
} from '../client-choices'

import Client = Schemas.Client

export interface ClientSettingsDraft {
  name: string
  clientId: string
  enabled: boolean
  directAccessGrants: boolean
  deviceCodeGrant: boolean
  requirePkce: boolean
  accessTokenLifetime: number | null
  refreshTokenLifetime: number | null
  idTokenLifetime: number | null
  temporaryTokenLifetime: number | null
}

export interface ClientSettingsTabProps {
  client: Client
  draft: ClientSettingsDraft
  errors: { clientId?: string; name?: string }
  dirtyCount: number
  redirectUris: string[]
  redirectUriError?: string
  webOrigins: string[]
  webOriginError?: string
  postLogoutRedirectUris: string[]
  postLogoutRedirectUriError?: string
  onDraftChange: (patch: Partial<ClientSettingsDraft>) => void
  onRedirectUrisChange: (next: string[]) => void
  onWebOriginsChange: (next: string[]) => void
  onPostLogoutRedirectUrisChange: (next: string[]) => void
  onDiscard: () => void
  onSave: () => void
  onDelete: () => void
}

const LIFETIMES = [
  {
    key: 'accessTokenLifetime',
    label: 'Access token lifetime',
    description: 'How long access tokens remain valid.',
  },
  {
    key: 'refreshTokenLifetime',
    label: 'Refresh token lifetime',
    description: 'How long refresh tokens remain valid.',
  },
  {
    key: 'idTokenLifetime',
    label: 'ID token lifetime',
    description: 'How long ID tokens remain valid.',
  },
  {
    key: 'temporaryTokenLifetime',
    label: 'Temporary token lifetime',
    description: 'How long temporary tokens (password reset, for instance) remain valid.',
  },
] as const

export default function ClientSettingsTab({
  client,
  draft,
  errors,
  dirtyCount,
  redirectUris,
  redirectUriError,
  webOrigins,
  webOriginError,
  postLogoutRedirectUris,
  postLogoutRedirectUriError,
  onDraftChange,
  onRedirectUrisChange,
  onWebOriginsChange,
  onPostLogoutRedirectUrisChange,
  onDiscard,
  onSave,
  onDelete,
}: ClientSettingsTabProps) {
  const isOidc = client.protocol === 'openid-connect'

  const pkceDescription = client.public_client
    ? 'Rejects authorization requests that carry no S256 code challenge (RFC 7636). The plain method is refused when this is on. Strongly recommended: this client cannot keep a secret safe.'
    : 'Rejects authorization requests that carry no S256 code challenge (RFC 7636). The plain method is refused when this is on.'

  return (
    <>
      <Section title='General' description='How this client is identified in the realm.'>
        <FieldRow
          label='Client name'
          description='Display name shown in the UI.'
          htmlFor='client-name'
        >
          <Input
            id='client-name'
            value={draft.name}
            onChange={(e) => onDraftChange({ name: e.target.value })}
            className='max-w-sm'
            aria-invalid={Boolean(errors.name)}
          />
          {errors.name && <p className='mt-1.5 text-fk-danger text-xs'>{errors.name}</p>}
        </FieldRow>

        <FieldRow
          label='Client ID'
          description='Unique identifier for this client. Applications send it on every request.'
          htmlFor='client-client-id'
        >
          <Input
            id='client-client-id'
            value={draft.clientId}
            onChange={(e) => onDraftChange({ clientId: e.target.value })}
            className='max-w-sm'
            aria-invalid={Boolean(errors.clientId)}
          />
          {errors.clientId && <p className='mt-1.5 text-fk-danger text-xs'>{errors.clientId}</p>}
        </FieldRow>

        <FieldRow label='Client enabled' description='Disabled clients cannot authenticate users.'>
          <ChoiceCards
            label='Client state'
            value={draft.enabled ? 'enabled' : 'disabled'}
            onChange={(v: ClientState) => onDraftChange({ enabled: v === 'enabled' })}
            options={stateChoices}
          />
        </FieldRow>
      </Section>

      <Section
        title='Capability config'
        description='Which authentication flows this client is allowed to take.'
      >
        <FieldRow
          label='Client authentication'
          description='Set at creation and final: a client cannot move between confidential and public afterwards.'
        >
          <ChoiceCards
            label='Client authentication'
            value={client.public_client ? 'public' : 'confidential'}
            onChange={() => undefined}
            options={lockedAuthenticationChoices}
          />
        </FieldRow>

        {isOidc && (
          <>
            <FieldRow
          label='Direct access grants'
          description='Allows exchanging user credentials directly for tokens. Use only for trusted clients.'
        >
          <SwitchField
            checked={draft.directAccessGrants}
            onCheckedChange={(v) => onDraftChange({ directAccessGrants: v })}
          />
        </FieldRow>

        <FieldRow
          label='OAuth 2.0 device authorization grant'
          description='Lets browserless clients (CLI, IoT, TVs) initiate a device flow against this client. Disable unless this client really needs it.'
        >
          <SwitchField
            checked={draft.deviceCodeGrant}
            onCheckedChange={(v) => onDraftChange({ deviceCodeGrant: v })}
          />
        </FieldRow>

        <FieldRow label='Require PKCE' description={pkceDescription}>
          <SwitchField
            checked={draft.requirePkce}
            onCheckedChange={(v) => onDraftChange({ requirePkce: v })}
            onLabel='Required'
            offLabel='Optional'
          />
        </FieldRow>
          </>
        )}
      </Section>

      {isOidc && (
      <Section
        title='Access'
        description='Addresses this client may be redirected to, and origins it may call from.'
      >
        <FieldRow
          label='Redirect URIs'
          description='Allowed redirect URIs after authentication. Every entry is saved as you add it.'
        >
          <ChipInput
            values={redirectUris}
            onChange={onRedirectUrisChange}
            placeholder='https://app.example.com/callback'
            emptyHint='No URI — the authorization code flow will fail.'
          />
          {redirectUriError && (
            <p className='mt-1.5 text-xs text-fk-danger'>{redirectUriError}</p>
          )}
        </FieldRow>

        <FieldRow
          label='Web origins'
          description='Origins this application may call FerrisKey from in a browser. Enter + to derive them from the redirect URIs above — literal ones only, regex patterns are skipped. Removing one takes a few minutes to clear browser preflight caches, and up to 30 seconds on the other API replicas.'
        >
          <ChipInput
            values={webOrigins}
            onChange={onWebOriginsChange}
            placeholder='https://app.example.com'
            emptyHint='No origin — browser calls from another host will be refused.'
          />
          {webOriginError && <p className='mt-1.5 text-xs text-fk-danger'>{webOriginError}</p>}
        </FieldRow>
      </Section>
      )}

      {isOidc && (
      <Section title='Logout' description='Where the user lands once the session is closed.'>
        <FieldRow
          label='Post-logout redirect URIs'
          description='Allowed redirect URIs after logout. Without an entry, logout ends on the FerrisKey page.'
        >
          <ChipInput
            values={postLogoutRedirectUris}
            onChange={onPostLogoutRedirectUrisChange}
            placeholder='https://app.example.com'
            emptyHint='No exit URI declared.'
          />
          {postLogoutRedirectUriError && (
            <p className='mt-1.5 text-xs text-fk-danger'>{postLogoutRedirectUriError}</p>
          )}
        </FieldRow>
      </Section>
      )}

      <Section
        title='Token lifetimes'
        description='Leave empty to inherit the realm default value.'
      >
        {LIFETIMES.map((lifetime) => (
          <FieldRow key={lifetime.key} label={lifetime.label} description={lifetime.description}>
            <DurationInput
              label={lifetime.label}
              value={draft[lifetime.key]}
              onChange={(seconds) => onDraftChange({ [lifetime.key]: seconds })}
              nullable
            />
          </FieldRow>
        ))}
      </Section>

      <DangerZone
        resourceName={client.name || client.client_id}
        label='Delete this client'
        description='Once deleted, all associated tokens, roles, and configurations will be permanently removed.'
        buttonLabel='Delete client'
        confirmTitle='Delete client'
        confirmDescription={`This will permanently delete the client "${client.name || client.client_id}" and all its associated data.`}
        onConfirm={onDelete}
      />

      <SaveBar
        show={dirtyCount > 0}
        title={`${dirtyCount} unsaved change${dirtyCount > 1 ? 's' : ''}`}
        description='Review the client before applying the changes.'
        onCancel={onDiscard}
        cancelLabel='Discard'
        actions={[{ label: 'Save changes', onClick: onSave }]}
      />
    </>
  )
}
