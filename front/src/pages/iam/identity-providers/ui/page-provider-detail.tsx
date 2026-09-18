import { ArrowLeft, Power, PowerOff } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/kit/button'
import { Input } from '@/components/ui/input'
import SaveBar from '@/components/kit/save-bar'
import { DangerZone } from '@/components/kit/danger-zone'
import { ChoiceCards, DetailHeader, FieldRow, PageShell, Pill, Section, StatusDot } from '@/components/kit'
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

const SECRET_MASK = '\u2022'.repeat(8)

const PROVIDER_STATE_ENABLED = 'enabled' as const
const PROVIDER_STATE_DISABLED = 'disabled' as const

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
  const { t } = useTranslation('identity-provider')

  const enabledChoices: Choice<'enabled' | 'disabled'>[] = [
    {
      value: PROVIDER_STATE_ENABLED,
      label: t('detail.enabled.on.label'),
      description: t('detail.enabled.on.description'),
      icon: Power,
    },
    {
      value: PROVIDER_STATE_DISABLED,
      label: t('detail.enabled.off.label'),
      description: t('detail.enabled.off.description'),
      icon: PowerOff,
    },
  ]

  const backButton = (
    <Button variant='ghost' size='sm' className='-ml-2 mb-2 text-neutral-500 dark:text-neutral-400' onClick={onBack}>
      <ArrowLeft className='size-3.5' />
      {t('detail.back')}
    </Button>
  )

  if (isLoading) {
    return (
      <PageShell>
        <div className='h-4 w-32 animate-pulse rounded bg-neutral-100 dark:bg-fk-raised' />
        <div className='mt-4 flex items-center gap-3'>
          <div className='size-15 animate-pulse rounded-md bg-neutral-100 dark:bg-fk-raised' />
          <div className='space-y-2'>
            <div className='h-5 w-48 animate-pulse rounded bg-neutral-100 dark:bg-fk-raised' />
            <div className='h-4 w-32 animate-pulse rounded bg-neutral-100 dark:bg-fk-raised' />
          </div>
        </div>
      </PageShell>
    )
  }

  if (!provider) {
    return (
      <PageShell>
        {backButton}
        <div className={cn(tokens.surface.panel, 'grid place-items-center px-6 py-16')}>
          <p className='text-sm font-medium text-neutral-700 dark:text-neutral-300'>
            {t('detail.not_found.title')}
          </p>
          <p className='mt-1 max-w-sm text-center text-sm text-neutral-500 dark:text-neutral-400'>
            {t('detail.not_found.hint')}
          </p>
        </div>
      </PageShell>
    )
  }

  const status = providerStatus(provider)
  const config = providerConfig(provider)
  const configEntries = Object.entries(config)

  const metadataRows = [
    { key: 'detail.metadata.provider_id', value: provider.provider_id },
    {
      key: 'detail.metadata.first_broker_flow',
      value: provider.first_broker_login_flow_alias || t('not_set'),
    },
    {
      key: 'detail.metadata.post_broker_flow',
      value: provider.post_broker_login_flow_alias || t('not_set'),
    },
  ]

  return (
    <PageShell>
      <DetailHeader
        onBack={onBack}
        backLabel={t('detail.back')}
        icon={<ProviderTile providerId={provider.provider_id} size='md' />}
        title={displayName || provider.alias}
        pills={
          <>
            <Pill mono>{provider.alias}</Pill>
            <Pill tone='violet' mono>
              {providerTypeLabel(provider.provider_id)}
            </Pill>
            <Pill tone={enabled ? 'success' : 'neutral'}>
              <StatusDot on={enabled} />
              {enabled ? t('badge.enabled') : t('badge.disabled')}
            </Pill>
            <ProviderStatusPill health={status.health} label={status.label} />
          </>
        }
        meta={
          <dl className='shrink-0 text-right text-xs text-neutral-500 dark:text-neutral-400'>
            <dt className='sr-only'>{t('detail.internal_id')}</dt>
            <dd className='font-mono-ui text-[11px] text-neutral-400 dark:text-neutral-500'>
              {provider.internal_id}
            </dd>
          </dl>
        }
      />

      {status.health !== 'healthy' && (
        <div
          className={cn(
            'mt-4 rounded-sm border px-3 py-2 text-[13px] text-neutral-700 dark:text-neutral-300',
            status.health === 'error'
              ? 'border-fk-danger-border bg-fk-danger-soft/40'
              : 'border-fk-amber-border bg-fk-amber-soft/50'
          )}
        >
          {status.detail}
        </div>
      )}

      <div className={cn('mt-5', tokens.page.blockGap)}>
        <Section title={t('detail.general.title')} description={t('detail.general.description')}>
          <FieldRow
            label={t('detail.alias.label')}
            description={t('detail.alias.description')}
            htmlFor='provider-alias'
          >
            <Input
              id='provider-alias'
              value={provider.alias}
              disabled
              className='max-w-sm'
            />
          </FieldRow>

          <FieldRow
            label={t('detail.display_name.label')}
            description={t('detail.display_name.description')}
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
            label={t('detail.enabled.label')}
            description={t('detail.enabled.description')}
          >
            <ChoiceCards
              label={t('detail.enabled.choices_label')}
              value={enabled ? PROVIDER_STATE_ENABLED : PROVIDER_STATE_DISABLED}
              onChange={(value) => onEnabledChange(value === PROVIDER_STATE_ENABLED)}
              options={enabledChoices}
            />
          </FieldRow>

          <FieldRow
            label={t('redirect_uri.label')}
            description={t('detail.redirect_uri_description')}
          >
            <CopyValue
              value={callbackUrl}
              copyLabel={t('redirect_uri.copy')}
              className='max-w-lg'
            />
          </FieldRow>
        </Section>

        <Section
          title={t('detail.config.title')}
          description={t('detail.config.description')}
          contained={configEntries.length > 0}
        >
          {configEntries.length > 0 ? (
            configEntries.map(([key, value]) => (
                <div
                  key={key}
                  className='grid gap-x-8 gap-y-1 py-3 md:grid-cols-[minmax(0,20rem)_minmax(0,1fr)]'
                >
                  <p className='text-sm font-medium text-neutral-900 dark:text-neutral-100'>
                    {humanizeConfigKey(key)}
                  </p>
                  <div className='min-w-0'>
                    {isSecretKey(key) ? (
                      <span
                        className='font-mono-ui text-xs text-neutral-400 dark:text-neutral-500'
                        title={t('detail.config.secret_title')}
                      >
                        {SECRET_MASK}
                      </span>
                    ) : (
                      <span className='block break-all font-mono-ui text-xs text-neutral-700 dark:text-neutral-300'>
                        {value === null || value === undefined || value === ''
                          ? t('not_set')
                          : String(value)}
                      </span>
                    )}
                  </div>
                </div>
            ))
          ) : (
            <p className='rounded-sm border border-dashed border-fk-line px-4 py-3 text-xs text-neutral-500 dark:text-neutral-400'>
              {t('detail.config.empty')}
            </p>
          )}
        </Section>

        <Section title={t('detail.metadata.title')} description={t('detail.metadata.description')}>
          {metadataRows.map(({ key, value }) => (
            <div
              key={key}
              className='grid gap-x-8 gap-y-1 py-3 md:grid-cols-[minmax(0,20rem)_minmax(0,1fr)]'
            >
              <p className='text-sm font-medium text-neutral-900 dark:text-neutral-100'>
              {t(key)}
            </p>
              <div className='min-w-0 break-all font-mono-ui text-xs text-neutral-500 dark:text-neutral-400'>
                {value}
              </div>
            </div>
          ))}
        </Section>

        <DangerZone
        resourceName={provider.display_name || provider.alias}
          label={t('detail.danger.label')}
          description={t('detail.danger.description')}
          buttonLabel={t('detail.danger.button')}
          confirmTitle={t('detail.danger.confirm_title')}
          confirmDescription={t('detail.danger.confirm_description', {
            name: providerName(provider),
          })}
          onConfirm={onDelete}
        />
      </div>

      <SaveBar
        show={dirtyCount > 0}
        title={t('detail.save.dirty', { count: dirtyCount })}
        description={t('detail.save.description')}
        onCancel={onDiscard}
        cancelLabel={t('detail.save.discard')}
        actions={[{ label: t('detail.save.submit'), onClick: onSave }]}
      />
    </PageShell>
  )
}
