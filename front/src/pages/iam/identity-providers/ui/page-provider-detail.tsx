import { ArrowLeft, Eye, EyeOff, Power, PowerOff } from 'lucide-react'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/kit/button'
import { Input } from '@/components/ui/input'
import {
  InputGroup,
  InputGroupAddon,
  InputGroupButton,
  InputGroupInput,
} from '@/components/ui/input-group'
import { Switch } from '@/components/ui/switch'
import SaveBar from '@/components/kit/save-bar'
import { DangerZone } from '@/components/kit/danger-zone'
import {
  ChipInput,
  ChoiceCards,
  DetailHeader,
  FieldRow,
  PageShell,
  Pill,
  Section,
  StatusDot,
} from '@/components/kit'
import type { Choice } from '@/components/kit'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'
import { Schemas } from '@/api/api.client'
import {
  humanizeConfigKey,
  isSecretKey,
  providerName,
  providerStatus,
  providerTypeLabel,
} from '../provider-status'
import {
  extraConfigEntries,
  type ProviderDraft,
  type ProviderDraftErrors,
} from '../provider-update-body'
import ProviderTile from './provider-tile'
import ProviderStatusPill from './provider-status-pill'
import CopyValue from './copy-value'

import IdentityProvider = Schemas.IdentityProviderResponse

export interface PageProviderDetailProps {
  provider?: IdentityProvider
  isLoading: boolean
  draft: ProviderDraft
  errors: ProviderDraftErrors
  callbackUrl: string
  dirtyCount: number
  canSave: boolean
  onChange: (patch: Partial<ProviderDraft>) => void
  onBack: () => void
  onDiscard: () => void
  onSave: () => void
  onDelete: () => void
}

const SECRET_MASK = '\u2022'.repeat(8)

const SCOPE_PLACEHOLDER = 'openid'

const PROVIDER_STATE_ENABLED = 'enabled' as const
const PROVIDER_STATE_DISABLED = 'disabled' as const

export default function PageProviderDetail({
  provider,
  isLoading,
  draft,
  errors,
  callbackUrl,
  dirtyCount,
  canSave,
  onChange,
  onBack,
  onDiscard,
  onSave,
  onDelete,
}: PageProviderDetailProps) {
  const { t } = useTranslation('identity-provider')
  const [showSecret, setShowSecret] = useState(false)

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
  const extraEntries = extraConfigEntries(provider.config)

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
        title={draft.displayName || provider.alias}
        pills={
          <>
            <Pill mono>{provider.alias}</Pill>
            <Pill tone='violet' mono>
              {providerTypeLabel(provider.provider_id)}
            </Pill>
            <Pill tone={draft.enabled ? 'success' : 'neutral'}>
              <StatusDot on={draft.enabled} />
              {draft.enabled ? t('badge.enabled') : t('badge.disabled')}
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
              value={draft.displayName}
              onChange={(e) => onChange({ displayName: e.target.value })}
              className='max-w-sm'
            />
          </FieldRow>

          <FieldRow
            label={t('detail.enabled.label')}
            description={t('detail.enabled.description')}
          >
            <ChoiceCards
              label={t('detail.enabled.choices_label')}
              value={draft.enabled ? PROVIDER_STATE_ENABLED : PROVIDER_STATE_DISABLED}
              onChange={(value) => onChange({ enabled: value === PROVIDER_STATE_ENABLED })}
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

        <Section title={t('detail.config.title')} description={t('detail.config.description')}>
          <FieldRow
            label={t('detail.client_id.label')}
            description={t('detail.client_id.description')}
            htmlFor='provider-client-id'
          >
            <Input
              id='provider-client-id'
              value={draft.clientId}
              onChange={(e) => onChange({ clientId: e.target.value })}
              className='max-w-sm'
              aria-invalid={Boolean(errors.clientId)}
            />
            {errors.clientId && (
              <p className='mt-1.5 text-xs text-fk-danger'>{t(errors.clientId)}</p>
            )}
          </FieldRow>

          <FieldRow
            label={t('detail.client_secret.label')}
            description={t('detail.client_secret.description')}
            htmlFor='provider-client-secret'
          >
            <InputGroup className='max-w-sm'>
              <InputGroupInput
                id='provider-client-secret'
                type={showSecret ? 'text' : 'password'}
                value={draft.clientSecret}
                placeholder={SECRET_MASK}
                onChange={(e) => onChange({ clientSecret: e.target.value })}
                className='font-mono-ui'
              />
              <InputGroupAddon align='inline-end'>
                <InputGroupButton
                  size='icon-xs'
                  aria-label={
                    showSecret ? t('detail.client_secret.hide') : t('detail.client_secret.show')
                  }
                  onClick={() => setShowSecret((value) => !value)}
                >
                  {showSecret ? <EyeOff /> : <Eye />}
                </InputGroupButton>
              </InputGroupAddon>
            </InputGroup>
          </FieldRow>

          <FieldRow
            label={t('detail.authorization_url.label')}
            description={t('detail.authorization_url.description')}
            htmlFor='provider-authorization-url'
          >
            <Input
              id='provider-authorization-url'
              value={draft.authorizationUrl}
              onChange={(e) => onChange({ authorizationUrl: e.target.value })}
              className='max-w-lg'
              aria-invalid={Boolean(errors.authorizationUrl)}
            />
            {errors.authorizationUrl && (
              <p className='mt-1.5 text-xs text-fk-danger'>{t(errors.authorizationUrl)}</p>
            )}
          </FieldRow>

          <FieldRow
            label={t('detail.token_url.label')}
            description={t('detail.token_url.description')}
            htmlFor='provider-token-url'
          >
            <Input
              id='provider-token-url'
              value={draft.tokenUrl}
              onChange={(e) => onChange({ tokenUrl: e.target.value })}
              className='max-w-lg'
              aria-invalid={Boolean(errors.tokenUrl)}
            />
            {errors.tokenUrl && (
              <p className='mt-1.5 text-xs text-fk-danger'>{t(errors.tokenUrl)}</p>
            )}
          </FieldRow>

          <FieldRow
            label={t('detail.userinfo_url.label')}
            description={t('detail.userinfo_url.description')}
            htmlFor='provider-userinfo-url'
          >
            <Input
              id='provider-userinfo-url'
              value={draft.userinfoUrl}
              onChange={(e) => onChange({ userinfoUrl: e.target.value })}
              placeholder={t('not_set')}
              className='max-w-lg'
              aria-invalid={Boolean(errors.userinfoUrl)}
            />
            {errors.userinfoUrl && (
              <p className='mt-1.5 text-xs text-fk-danger'>{t(errors.userinfoUrl)}</p>
            )}
          </FieldRow>

          <FieldRow label={t('detail.scopes.label')} description={t('detail.scopes.description')}>
            <ChipInput
              values={draft.scopes}
              onChange={(scopes) => onChange({ scopes })}
              placeholder={SCOPE_PLACEHOLDER}
              emptyHint={t('detail.scopes.empty_hint')}
            />
          </FieldRow>

          <FieldRow label={t('detail.pkce.label')} description={t('detail.pkce.description')}>
            <div className='flex items-center gap-2.5'>
              <Switch
                id='provider-use-pkce'
                checked={draft.usePkce}
                onCheckedChange={(usePkce) => onChange({ usePkce })}
              />
              <label
                htmlFor='provider-use-pkce'
                className='text-xs text-neutral-600 dark:text-neutral-400'
              >
                {draft.usePkce ? t('detail.pkce.enabled') : t('detail.pkce.disabled')}
              </label>
            </div>
          </FieldRow>
        </Section>

        {extraEntries.length > 0 && (
          <Section
            title={t('detail.config.extra_title')}
            description={t('detail.config.extra_description')}
            contained
          >
            {extraEntries.map(([key, value]) => (
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
            ))}
          </Section>
        )}

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
        actions={[{ label: t('detail.save.submit'), onClick: onSave, disabled: !canSave }]}
      />
    </PageShell>
  )
}
