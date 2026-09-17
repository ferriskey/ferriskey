import { useTranslation } from 'react-i18next'
import { Input } from '@/components/ui/input'
import { DurationInput } from '@/components/ui/duration-input'
import SaveBar from '@/components/kit/save-bar'
import { DangerZone } from '@/components/kit/danger-zone'
import { ChipInput, ChoiceCards, FieldRow, Section, SwitchField } from '@/components/kit'
import { Schemas } from '@/api/api.client'
import {
  clientAuthenticationOf,
  clientStateOf,
  isEnabledState,
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
  { key: 'accessTokenLifetime', field: 'access_token' },
  { key: 'refreshTokenLifetime', field: 'refresh_token' },
  { key: 'idTokenLifetime', field: 'id_token' },
  { key: 'temporaryTokenLifetime', field: 'temporary_token' },
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
  const { t } = useTranslation('client')
  const isOidc = client.protocol === 'openid-connect'

  const pkceDescription = client.public_client
    ? t('settings.capability.pkce.description_public')
    : t('settings.capability.pkce.description')

  return (
    <>
      <Section title={t('settings.general.title')} description={t('settings.general.description')}>
        <FieldRow
          label={t('settings.general.name.label')}
          description={t('settings.general.name.description')}
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
          label={t('settings.general.client_id.label')}
          description={t('settings.general.client_id.description')}
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

        <FieldRow
          label={t('settings.general.enabled.label')}
          description={t('settings.general.enabled.description')}
        >
          <ChoiceCards
            label={t('settings.general.state_picker')}
            value={clientStateOf(draft.enabled)}
            onChange={(v: ClientState) => onDraftChange({ enabled: isEnabledState(v) })}
            options={stateChoices(t)}
          />
        </FieldRow>
      </Section>

      <Section
        title={t('settings.capability.title')}
        description={t('settings.capability.description')}
      >
        <FieldRow
          label={t('settings.capability.authentication.label')}
          description={t('settings.capability.authentication.description')}
        >
          <ChoiceCards
            label={t('settings.capability.authentication.label')}
            value={clientAuthenticationOf(client.public_client)}
            onChange={() => undefined}
            options={lockedAuthenticationChoices(t)}
          />
        </FieldRow>

        {isOidc && (
          <>
            <FieldRow
          label={t('settings.capability.direct_access_grants.label')}
          description={t('settings.capability.direct_access_grants.description')}
        >
          <SwitchField
            checked={draft.directAccessGrants}
            onCheckedChange={(v) => onDraftChange({ directAccessGrants: v })}
          />
        </FieldRow>

        <FieldRow
          label={t('settings.capability.device_code_grant.label')}
          description={t('settings.capability.device_code_grant.description')}
        >
          <SwitchField
            checked={draft.deviceCodeGrant}
            onCheckedChange={(v) => onDraftChange({ deviceCodeGrant: v })}
          />
        </FieldRow>

        <FieldRow label={t('settings.capability.pkce.label')} description={pkceDescription}>
          <SwitchField
            checked={draft.requirePkce}
            onCheckedChange={(v) => onDraftChange({ requirePkce: v })}
            onLabel={t('settings.capability.pkce.required')}
            offLabel={t('settings.capability.pkce.optional')}
          />
        </FieldRow>
          </>
        )}
      </Section>

      {isOidc && (
      <Section title={t('settings.access.title')} description={t('settings.access.description')}>
        <FieldRow
          label={t('settings.access.redirect_uris.label')}
          description={t('settings.access.redirect_uris.description')}
        >
          <ChipInput
            values={redirectUris}
            onChange={onRedirectUrisChange}
            placeholder={t('settings.access.redirect_uris.placeholder')}
            emptyHint={t('settings.access.redirect_uris.empty')}
          />
          {redirectUriError && (
            <p className='mt-1.5 text-xs text-fk-danger'>{redirectUriError}</p>
          )}
        </FieldRow>

        <FieldRow
          label={t('settings.access.web_origins.label')}
          description={t('settings.access.web_origins.description')}
        >
          <ChipInput
            values={webOrigins}
            onChange={onWebOriginsChange}
            placeholder={t('settings.access.web_origins.placeholder')}
            emptyHint={t('settings.access.web_origins.empty')}
          />
          {webOriginError && <p className='mt-1.5 text-xs text-fk-danger'>{webOriginError}</p>}
        </FieldRow>
      </Section>
      )}

      {isOidc && (
      <Section title={t('settings.logout.title')} description={t('settings.logout.description')}>
        <FieldRow
          label={t('settings.logout.post_logout_redirect_uris.label')}
          description={t('settings.logout.post_logout_redirect_uris.description')}
        >
          <ChipInput
            values={postLogoutRedirectUris}
            onChange={onPostLogoutRedirectUrisChange}
            placeholder={t('settings.logout.post_logout_redirect_uris.placeholder')}
            emptyHint={t('settings.logout.post_logout_redirect_uris.empty')}
          />
          {postLogoutRedirectUriError && (
            <p className='mt-1.5 text-xs text-fk-danger'>{postLogoutRedirectUriError}</p>
          )}
        </FieldRow>
      </Section>
      )}

      <Section
        title={t('settings.lifetimes.title')}
        description={t('settings.lifetimes.description')}
      >
        {LIFETIMES.map((lifetime) => {
          const label = t(`settings.lifetimes.${lifetime.field}.label`)

          return (
            <FieldRow
              key={lifetime.key}
              label={label}
              description={t(`settings.lifetimes.${lifetime.field}.description`)}
            >
              <DurationInput
                label={label}
                value={draft[lifetime.key]}
                onChange={(seconds) => onDraftChange({ [lifetime.key]: seconds })}
                nullable
              />
            </FieldRow>
          )
        })}
      </Section>

      <DangerZone
        resourceName={client.name || client.client_id}
        label={t('settings.danger.label')}
        description={t('settings.danger.description')}
        buttonLabel={t('settings.danger.button')}
        confirmTitle={t('settings.danger.confirm_title')}
        confirmDescription={t('settings.danger.confirm_description', {
          name: client.name || client.client_id,
        })}
        onConfirm={onDelete}
      />

      <SaveBar
        show={dirtyCount > 0}
        title={t('shared.unsaved', { count: dirtyCount })}
        description={t('settings.save.description')}
        onCancel={onDiscard}
        cancelLabel={t('shared.discard')}
        actions={[{ label: t('shared.save'), onClick: onSave }]}
      />
    </>
  )
}
