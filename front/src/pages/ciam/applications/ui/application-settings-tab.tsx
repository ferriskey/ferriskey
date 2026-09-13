import { Input } from '@/components/ui/input'
import { DurationInput } from '@/components/ui/duration-input'
import SaveBar from '@/components/kit/save-bar'
import { DangerZone } from '@/components/kit/danger-zone'
import { ChipInput, ChoiceCards, FieldRow, Pill, Section, SwitchField } from '@/components/kit'
import type { Choice } from '@/components/kit'
import { Power, PowerOff } from 'lucide-react'
import { Schemas } from '@/api/api.client'
import { APPLICATION_FIELDS, applicationTypeMeta, inferApplicationType } from '../application-types'

import Client = Schemas.Client

type ApplicationState = 'enabled' | 'disabled'

const stateChoices: Choice<ApplicationState>[] = [
  {
    value: 'enabled',
    label: 'Enabled',
    description: 'Your users can sign in through this application.',
    icon: Power,
  },
  {
    value: 'disabled',
    label: 'Disabled',
    description: 'Every sign-in is refused. The configuration is kept.',
    icon: PowerOff,
  },
]

export interface ApplicationSettingsDraft {
  name: string
  enabled: boolean
  directAccessGrants: boolean
  deviceCodeGrant: boolean
  requirePkce: boolean
  accessTokenLifetime: number | null
  refreshTokenLifetime: number | null
  idTokenLifetime: number | null
  temporaryTokenLifetime: number | null
}

export interface ApplicationSettingsTabProps {
  application: Client
  draft: ApplicationSettingsDraft
  errors: { name?: string }
  dirtyCount: number
  callbacks: string[]
  callbackError?: string
  origins: string[]
  originError?: string
  onDraftChange: (patch: Partial<ApplicationSettingsDraft>) => void
  onCallbacksChange: (next: string[]) => void
  onOriginsChange: (next: string[]) => void
  onDiscard: () => void
  onSave: () => void
  onDelete: () => void
}

const LIFETIMES = [
  {
    key: 'accessTokenLifetime',
    label: 'Access token',
    description: 'How long a token is accepted by your API before it must be refreshed.',
  },
  {
    key: 'refreshTokenLifetime',
    label: 'Refresh token',
    description: 'How long a signed-in user stays signed in without typing their password again.',
  },
  {
    key: 'idTokenLifetime',
    label: 'ID token',
    description: 'How long the identity token describing the user stays valid.',
  },
  {
    key: 'temporaryTokenLifetime',
    label: 'Temporary token',
    description: 'How long a one-off link, such as a password reset, keeps working.',
  },
] as const

export default function ApplicationSettingsTab({
  application,
  draft,
  errors,
  dirtyCount,
  callbacks,
  callbackError,
  origins,
  originError,
  onDraftChange,
  onCallbacksChange,
  onOriginsChange,
  onDiscard,
  onSave,
  onDelete,
}: ApplicationSettingsTabProps) {
  const type = inferApplicationType(application)
  const meta = applicationTypeMeta(type)
  const fields = APPLICATION_FIELDS[type]

  const pkceDescription = meta.holdsSecret
    ? 'Refuses a sign-in that carries no S256 code challenge (RFC 7636). The plain method is refused too.'
    : 'Refuses a sign-in that carries no S256 code challenge (RFC 7636). Strongly recommended: this application ships to your users, so it cannot keep a secret.'

  return (
    <>
      <Section title='General' description='How this application appears and whether it works.'>
        <FieldRow
          label='Application name'
          description='Shown to your users on the sign-in page.'
          htmlFor='application-name'
        >
          <Input
            id='application-name'
            value={draft.name}
            onChange={(e) => onDraftChange({ name: e.target.value })}
            className='max-w-sm'
            aria-invalid={Boolean(errors.name)}
          />
          {errors.name && <p className='mt-1.5 text-xs text-fk-danger'>{errors.name}</p>}
        </FieldRow>

        <FieldRow
          label='Client ID'
          description='Sent on every OAuth request and hardcoded in your application. Changing it would break every running integration, so it is fixed here.'
        >
          <p className='font-mono-ui text-xs text-neutral-500 dark:text-neutral-400'>
            {application.client_id}
          </p>
        </FieldRow>

        <FieldRow
          label='Application type'
          description='Chosen at creation. It decides the sign-in flow and whether a secret exists, so it cannot be switched afterwards — create a new application instead.'
        >
          <div className='flex flex-wrap items-center gap-2'>
            <Pill tone={meta.tone}>{meta.label}</Pill>
            <span className='font-mono-ui text-xs text-neutral-500 dark:text-neutral-400'>
              {meta.flow}
            </span>
          </div>
        </FieldRow>

        <FieldRow
          label='Availability'
          description='Disabling stops every sign-in immediately, including for users already using the application.'
        >
          <ChoiceCards
            label='Application state'
            value={draft.enabled ? 'enabled' : 'disabled'}
            onChange={(v: ApplicationState) => onDraftChange({ enabled: v === 'enabled' })}
            options={stateChoices}
          />
        </FieldRow>
      </Section>

      {meta.usesAuthorizationCode && (
        <Section
          title='Where users come back'
          description='Addresses FerrisKey is allowed to send the user to. Every entry is saved as you add it.'
        >
          <FieldRow label={fields.callbacksLabel} description={fields.callbacksHint}>
            <ChipInput
              values={callbacks}
              onChange={onCallbacksChange}
              placeholder={fields.callbacksPlaceholder}
              emptyHint='No callback URL — sign-in cannot complete.'
            />
            {callbackError && <p className='mt-1.5 text-xs text-fk-danger'>{callbackError}</p>}
          </FieldRow>

          <FieldRow
            label='Allowed web origins'
            description='Origins allowed to call FerrisKey from a browser. Scheme, host and port only — no path. Enter + to derive them from the callback URLs above; regex patterns are skipped. Removing one takes a few minutes to clear browser preflight caches, and up to 30 seconds on the other API replicas.'
          >
            <ChipInput
              values={origins}
              onChange={onOriginsChange}
              placeholder='https://app.acme.com'
              emptyHint='No origin — browser calls from another host will be refused.'
            />
            {originError && <p className='mt-1.5 text-xs text-fk-danger'>{originError}</p>}
          </FieldRow>
        </Section>
      )}

      <Section
        title='Sign-in methods'
        description='Ways this application is allowed to obtain a token.'
      >
        <FieldRow
          label='Password exchange'
          description='Lets the application collect a username and password itself and swap them for tokens. It bypasses the FerrisKey sign-in page, so MFA and social sign-in never run. Leave off unless a legacy integration needs it.'
        >
          <SwitchField
            checked={draft.directAccessGrants}
            onCheckedChange={(v) => onDraftChange({ directAccessGrants: v })}
          />
        </FieldRow>

        <FieldRow
          label='Device authorization'
          description='Lets a browserless client (CLI, IoT, TV) show a code the user approves on another screen.'
        >
          <SwitchField
            checked={draft.deviceCodeGrant}
            onCheckedChange={(v) => onDraftChange({ deviceCodeGrant: v })}
          />
        </FieldRow>

        {meta.usesAuthorizationCode && (
          <FieldRow label='Require PKCE' description={pkceDescription}>
            <SwitchField
              checked={draft.requirePkce}
              onCheckedChange={(v) => onDraftChange({ requirePkce: v })}
              onLabel='Required'
              offLabel='Optional'
            />
          </FieldRow>
        )}
      </Section>

      <Section
        title='Session length'
        description='Leave a field empty to follow the realm default.'
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
        resourceName={application.name || application.client_id}
        label='Delete this application'
        description='Every token it issued stops being accepted and the integration breaks immediately. This cannot be undone.'
        buttonLabel='Delete application'
        confirmTitle='Delete application'
        confirmDescription={`This permanently deletes "${application.name || application.client_id}" and all of its configuration.`}
        onConfirm={onDelete}
      />

      <SaveBar
        show={dirtyCount > 0}
        title={`${dirtyCount} unsaved change${dirtyCount > 1 ? 's' : ''}`}
        description='Review the application before applying the changes.'
        onCancel={onDiscard}
        cancelLabel='Discard'
        actions={[{ label: 'Save changes', onClick: onSave }]}
      />
    </>
  )
}
