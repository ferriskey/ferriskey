import { ArrowLeft, Power, PowerOff } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import FloatingActionBar from '@/components/ui/floating-action-bar'
import { DangerZone } from '@/components/danger-zone'
import { ChoiceCards, FieldRow, Pill, Section, StatusDot } from '@/components/kit'
import type { Choice } from '@/components/kit'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'
import { Schemas } from '@/api/api.client'
import {
  humanizeConfigKey,
  isSecretKey,
  providerConfig,
  providerName,
  providerStatus,
  providerTypeLabel,
} from '../provider-status'
import ProviderTile from './provider-tile'
import ProviderStatusPill from './provider-status-pill'
import CopyValue from './copy-value'

import IdentityProvider = Schemas.IdentityProviderResponse

export interface PageProviderDetailProps {
  provider?: IdentityProvider
  isLoading: boolean
  displayName: string
  enabled: boolean
  callbackUrl: string
  dirtyCount: number
  onDisplayNameChange: (value: string) => void
  onEnabledChange: (value: boolean) => void
  onBack: () => void
  onDiscard: () => void
  onSave: () => void
  onDelete: () => void
}

const enabledChoices: Choice<'enabled' | 'disabled'>[] = [
  {
    value: 'enabled',
    label: 'Enabled',
    description: 'The provider appears on the login page of this realm.',
    icon: Power,
  },
  {
    value: 'disabled',
    label: 'Disabled',
    description: 'Hidden from the login page; already linked accounts stay linked.',
    icon: PowerOff,
  },
]

export default function PageProviderDetail({
  provider,
  isLoading,
  displayName,
  enabled,
  callbackUrl,
  dirtyCount,
  onDisplayNameChange,
  onEnabledChange,
  onBack,
  onDiscard,
  onSave,
  onDelete,
}: PageProviderDetailProps) {
  const container = cn('mx-auto', tokens.page.maxWidth, tokens.page.padding)

  const backButton = (
    <Button variant='ghost' size='sm' className='-ml-2 mb-2 text-neutral-500' onClick={onBack}>
      <ArrowLeft className='size-3.5' />
      Identity Providers
    </Button>
  )

  if (isLoading) {
    return (
      <div className={container}>
        <div className='h-4 w-32 animate-pulse rounded bg-neutral-100' />
        <div className='mt-4 flex items-center gap-3'>
          <div className='size-15 animate-pulse rounded-md bg-neutral-100' />
          <div className='space-y-2'>
            <div className='h-5 w-48 animate-pulse rounded bg-neutral-100' />
            <div className='h-4 w-32 animate-pulse rounded bg-neutral-100' />
          </div>
        </div>
      </div>
    )
  }

  if (!provider) {
    return (
      <div className={container}>
        {backButton}
        <div className={cn(tokens.surface.panel, 'grid place-items-center px-6 py-16')}>
          <p className='text-sm font-medium text-neutral-700'>Provider not found</p>
          <p className='mt-1 max-w-sm text-center text-sm text-neutral-500'>
            It may have been deleted, or it belongs to another realm.
          </p>
        </div>
      </div>
    )
  }

  const status = providerStatus(provider)
  const config = providerConfig(provider)
  const configEntries = Object.entries(config)

  return (
    <div className={container}>
      {backButton}

      <div className='flex flex-wrap items-start justify-between gap-4'>
        <div className='flex items-center gap-3'>
          <ProviderTile providerId={provider.provider_id} size='md' />
          <div className='min-w-0'>
            <h1 className={tokens.header.title}>{displayName || provider.alias}</h1>
            <div className='mt-1.5 flex flex-wrap items-center gap-2'>
              <Pill mono>{provider.alias}</Pill>
              <Pill tone='violet' mono>
                {providerTypeLabel(provider.provider_id)}
              </Pill>
              <Pill tone={enabled ? 'success' : 'neutral'}>
                <StatusDot on={enabled} />
                {enabled ? 'enabled' : 'disabled'}
              </Pill>
              <ProviderStatusPill health={status.health} label={status.label} />
            </div>
          </div>
        </div>

        <dl className='shrink-0 text-right text-xs text-neutral-500'>
          <dt className='sr-only'>Internal identifier</dt>
          <dd className='font-mono-ui text-[11px] text-neutral-400'>
            {provider.internal_id}
          </dd>
        </dl>
      </div>

      {status.health !== 'healthy' && (
        <div
          className={cn(
            'mt-4 rounded-sm border px-3 py-2 text-[13px] text-neutral-700',
            status.health === 'error'
              ? 'border-fk-danger-border bg-fk-danger-soft/40'
              : 'border-fk-amber-border bg-fk-amber-soft/50'
          )}
        >
          {status.detail}
        </div>
      )}

      <div className={cn('mt-5', tokens.page.blockGap)}>
        <Section title='General Settings' description='How this provider identifies itself in the realm.'>
          <FieldRow
            label='Alias'
            description='Unique identifier of the provider in this realm. Sealed at creation: it is part of the Redirect URI already declared at the provider, so changing it would break the callback.'
            htmlFor='provider-alias'
          >
            <Input
              id='provider-alias'
              value={provider.alias}
              disabled
              className='max-w-sm font-mono-ui'
            />
          </FieldRow>

          <FieldRow
            label='Display Name'
            description='Label of the button users click on the login page.'
            htmlFor='provider-display-name'
          >
            <Input
              id='provider-display-name'
              value={displayName}
              onChange={(e) => onDisplayNameChange(e.target.value)}
              className='max-w-sm'
            />
          </FieldRow>

          <FieldRow
            label='Enabled'
            description='Whether users may authenticate through this provider.'
          >
            <ChoiceCards
              label='Provider state'
              value={enabled ? 'enabled' : 'disabled'}
              onChange={(value) => onEnabledChange(value === 'enabled')}
              options={enabledChoices}
            />
          </FieldRow>

          <FieldRow
            label='Redirect URI'
            description='Declare this URL in the OAuth application of the provider.'
          >
            <CopyValue value={callbackUrl} label='the redirect URI' className='max-w-lg' />
          </FieldRow>
        </Section>

        <Section
          title='Configuration'
          description='Protocol settings, as the API returns them. Secrets never leave the server in clear text.'
          contained={configEntries.length > 0}
        >
          {configEntries.length > 0 ? (
            configEntries.map(([key, value]) => (
                <div
                  key={key}
                  className='grid gap-x-8 gap-y-1 py-3 md:grid-cols-[minmax(0,20rem)_minmax(0,1fr)]'
                >
                  <p className='text-sm font-medium text-neutral-900'>
                    {humanizeConfigKey(key)}
                  </p>
                  <div className='min-w-0'>
                    {isSecretKey(key) ? (
                      <span
                        className='font-mono-ui text-xs text-neutral-400'
                        title='The API never returns this value in clear text.'
                      >
                        ••••••••
                      </span>
                    ) : (
                      <span className='block break-all font-mono-ui text-xs text-neutral-700'>
                        {value === null || value === undefined || value === ''
                          ? 'Not set'
                          : String(value)}
                      </span>
                    )}
                  </div>
                </div>
            ))
          ) : (
            <p className='rounded-sm border border-dashed border-fk-line px-4 py-3 text-xs text-neutral-500'>
              No configuration setting recorded — this provider cannot broker a login.
            </p>
          )}
        </Section>

        <Section title='Metadata' description='System information, not editable.'>
          {(
            [
              ['Provider ID', provider.provider_id],
              ['First Broker Flow', provider.first_broker_login_flow_alias || 'Not set'],
              ['Post Broker Flow', provider.post_broker_login_flow_alias || 'Not set'],
            ] as const
          ).map(([label, value]) => (
            <div
              key={label}
              className='grid gap-x-8 gap-y-1 py-3 md:grid-cols-[minmax(0,20rem)_minmax(0,1fr)]'
            >
              <p className='text-sm font-medium text-neutral-900'>{label}</p>
              <div className='min-w-0 break-all font-mono-ui text-xs text-neutral-500'>
                {value}
              </div>
            </div>
          ))}
        </Section>

        <DangerZone
          label='Delete this identity provider'
          description='All associated data will be permanently removed. This action is irreversible.'
          buttonLabel='Delete provider'
          confirmTitle='Delete identity provider'
          confirmDescription={`This will permanently delete the provider "${providerName(provider)}" and all its associated data.`}
          onConfirm={onDelete}
        />
      </div>

      <FloatingActionBar
        show={dirtyCount > 0}
        title={`${dirtyCount} unsaved change${dirtyCount > 1 ? 's' : ''}`}
        description='Review the provider before applying the changes.'
        onCancel={onDiscard}
        cancelLabel='Discard'
        actions={[{ label: 'Save changes', onClick: onSave }]}
      />
    </div>
  )
}
