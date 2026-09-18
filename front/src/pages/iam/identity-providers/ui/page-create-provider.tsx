import { useMemo, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { ArrowLeft, ArrowRight, Check, Eye, EyeOff, Plus, Search } from 'lucide-react'
import { Button } from '@/components/kit/button'
import { Input } from '@/components/ui/input'
import {
  InputGroup,
  InputGroupAddon,
  InputGroupButton,
  InputGroupInput,
} from '@/components/ui/input-group'
import { ChipInput, FieldRow, PageShell, Pill, Section } from '@/components/kit'
import ProviderIcon from '@/components/provider-icon'
import {
  templateDescription,
  templateDisplayName,
  type ProviderTemplate,
} from '@/constants/identity-provider-templates'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'
import type { ProviderProtocol } from '../provider-status'
import ProviderSetupRail from './provider-setup-rail'

export interface CreateProviderValues {
  alias: string
  displayName: string
  clientId: string
  clientSecret: string
  authorizationUrl: string
  tokenUrl: string
  userinfoUrl: string
  scopes: string[]
}

export type CreateProviderErrors = Partial<Record<keyof CreateProviderValues, string>>

export interface PageCreateProviderProps {
  protocol: ProviderProtocol
  templates: ProviderTemplate[]
  template: ProviderTemplate | null
  step: number
  values: CreateProviderValues
  errors: CreateProviderErrors
  callbackUrl: string
  canContinue: boolean
  isPending: boolean
  onSelectTemplate: (template: ProviderTemplate) => void
  onChange: (patch: Partial<CreateProviderValues>) => void
  onNext: () => void
  onBack: () => void
  onSubmit: () => void
}

const CUSTOM_TEMPLATE_ID = 'custom'

const ALIAS_PLACEHOLDER = 'google-workspace'
const AUTHORIZATION_URL_PLACEHOLDER = 'https://provider.com/oauth/authorize'
const TOKEN_URL_PLACEHOLDER = 'https://provider.com/oauth/token'
const USERINFO_URL_PLACEHOLDER = 'https://provider.com/api/userinfo'
const SCOPE_PLACEHOLDER = 'openid'
const SECRET_MASK_CHARACTER = '\u2022'
const SECRET_MASK_LENGTH = 24

const steps = [
  { n: 1, titleKey: 'create.steps.provider' },
  { n: 2, titleKey: 'create.steps.configuration' },
  { n: 3, titleKey: 'create.steps.review' },
] as const

const categoryOrder: ProviderTemplate['category'][] = [
  'social',
  'enterprise',
  'developer',
  'custom',
]

function Stepper({ current }: { current: number }) {
  const { t } = useTranslation('identity-provider')

  return (
    <ol className='flex items-center gap-2'>
      {steps.map((step, index) => {
        const done = current > step.n
        const active = current === step.n

        return (
          <li key={step.n} className='flex items-center gap-2'>
            <span
              className={cn(
                'grid size-5 shrink-0 place-items-center rounded-full text-[11px] font-medium transition-colors',
                done && 'bg-fk-primary text-white',
                active && 'bg-fk-primary-soft text-fk-primary-text',
                !done && !active && 'bg-neutral-100 text-neutral-400 dark:bg-fk-raised dark:text-neutral-500'
              )}
            >
              {done ? <Check className='size-3' strokeWidth={3} /> : step.n}
            </span>
            <span
              className={cn(
                'text-xs',
                active
                  ? 'font-medium text-neutral-900 dark:text-neutral-100'
                  : done
                    ? 'text-neutral-600 dark:text-neutral-400'
                    : 'text-neutral-400 dark:text-neutral-500'
              )}
            >
              {t(step.titleKey)}
            </span>
            {index < steps.length - 1 && (
              <span
                className={cn('mx-1 h-px w-8', done ? 'bg-fk-primary-border' : 'bg-fk-line')}
              />
            )}
          </li>
        )
      })}
    </ol>
  )
}

export default function PageCreateProvider({
  protocol,
  templates,
  template,
  step,
  values,
  errors,
  callbackUrl,
  canContinue,
  isPending,
  onSelectTemplate,
  onChange,
  onNext,
  onBack,
  onSubmit,
}: PageCreateProviderProps) {
  const { t } = useTranslation('identity-provider')
  const [search, setSearch] = useState('')
  const [showSecret, setShowSecret] = useState(false)

  const matching = useMemo(() => {
    if (!search) return templates

    const query = search.toLowerCase()
    return templates.filter(
      (item) =>
        templateDisplayName(item).toLowerCase().includes(query) ||
        templateDescription(item).toLowerCase().includes(query)
    )
  }, [templates, search])

  const groups = useMemo(
    () =>
      categoryOrder
        .map((category) => ({
          category,
          items: matching.filter((item) => item.category === category),
        }))
        .filter((group) => group.items.length > 0),
    [matching]
  )

  const isCustom = template?.id === CUSTOM_TEMPLATE_ID
  const providerLabel = template ? templateDisplayName(template) : ''

  const reviewRows = [
    { key: 'create.review.template', value: providerLabel },
    { key: 'create.review.protocol', value: protocol },
    { key: 'create.alias.label', value: values.alias },
    { key: 'create.display_name.label', value: values.displayName || providerLabel },
    { key: 'create.client_id.label', value: values.clientId },
    {
      key: 'create.client_secret.label',
      value: SECRET_MASK_CHARACTER.repeat(
        Math.min(values.clientSecret.length, SECRET_MASK_LENGTH)
      ),
    },
    { key: 'redirect_uri.label', value: callbackUrl },
    { key: 'create.authorization_url.label', value: values.authorizationUrl },
    { key: 'create.token_url.label', value: values.tokenUrl },
    { key: 'create.userinfo_url.label', value: values.userinfoUrl || t('not_set') },
    { key: 'create.scopes.label', value: values.scopes.join(' ') || t('none') },
  ]

  return (
    <PageShell>
      <Button variant='ghost' size='sm' className='-ml-2 mb-2 text-neutral-500 dark:text-neutral-400' onClick={onBack}>
        <ArrowLeft className='size-3.5' />
        {t('create.back')}
      </Button>

      <div className={tokens.header.spacing}>
        <h1 className={tokens.header.title}>{t('create.title')}</h1>
        <p className='mt-0.5 text-sm text-neutral-500 dark:text-neutral-400'>
          {t('create.description', { protocol: protocol.toUpperCase() })}
        </p>
      </div>

      <div className={cn(tokens.surface.panel, 'mb-4 px-4 py-3')}>
        <Stepper current={step} />
      </div>

      {step === 1 && (
        <Section
          title={t('create.templates.title')}
          description={t('create.templates.description')}
          contained={false}
        >
          <div className='space-y-5'>
            <label className='relative flex h-8 max-w-sm items-center'>
              <Search className='pointer-events-none absolute left-2.5 size-3.5 text-neutral-400 dark:text-neutral-500' />
              <input
                type='search'
                value={search}
                onChange={(e) => setSearch(e.target.value)}
                placeholder={t('create.templates.search_placeholder')}
                className='h-full w-full rounded-md border border-fk-line bg-white dark:bg-fk-surface pl-8 pr-3 text-sm outline-none placeholder:text-neutral-400 focus:border-fk-primary-border focus:ring-2 focus:ring-fk-primary/15'
              />
            </label>

            {groups.length === 0 && (
              <p className='text-sm text-neutral-500 dark:text-neutral-400'>
                {t('create.templates.empty', { query: search })}
              </p>
            )}

            {groups.map((group) => (
              <div key={group.category}>
                <p className='pb-2 text-xs font-semibold uppercase tracking-wide text-neutral-500 dark:text-neutral-400'>
                  {t(`template_category.${group.category}`)}
                </p>
                <div className='grid gap-2 sm:grid-cols-2 xl:grid-cols-3'>
                  {group.items.map((item) => (
                    <button
                      key={item.id}
                      type='button'
                      onClick={() => onSelectTemplate(item)}
                      className={cn(
                        tokens.surface.panel,
                        'flex cursor-pointer items-center gap-3 p-3 text-left transition-colors hover:border-fk-primary-border hover:bg-fk-primary-soft/30',
                        'focus-visible:outline-none focus-visible:ring-3 focus-visible:ring-fk-primary/30',
                        item.id === template?.id && 'border-fk-primary-border bg-fk-primary-soft'
                      )}
                    >
                      <span className='grid size-9 shrink-0 place-items-center rounded-md border border-fk-line bg-white dark:bg-fk-surface'>
                        <ProviderIcon icon={item.icon} size='sm' />
                      </span>
                      <span className='min-w-0 flex-1'>
                        <span className='flex items-center gap-2'>
                          <span className='truncate text-xs font-medium text-neutral-900 dark:text-neutral-100'>
                            {templateDisplayName(item)}
                          </span>
                          <Pill tone={item.provider_type === 'oidc' ? 'violet' : 'amber'} mono>
                            {item.provider_type}
                          </Pill>
                        </span>
                        <span className='mt-0.5 block truncate text-xs text-neutral-500 dark:text-neutral-400'>
                          {templateDescription(item)}
                        </span>
                      </span>
                    </button>
                  ))}
                </div>
              </div>
            ))}
          </div>
        </Section>
      )}

      {step === 2 && template && (
        <div className='grid gap-4 lg:grid-cols-[minmax(0,1fr)_20rem]'>
          <div className={tokens.page.blockGap}>
            <Section
              title={t('create.credentials.title', { provider: providerLabel })}
              description={t('create.credentials.description')}
            >
              <FieldRow
                label={t('create.alias.label')}
                description={t('create.alias.description')}
                htmlFor='create-provider-alias'
              >
                <Input
                  id='create-provider-alias'
                  value={values.alias}
                  onChange={(e) => onChange({ alias: e.target.value })}
                  placeholder={ALIAS_PLACEHOLDER}
                  className='max-w-sm'
                  aria-invalid={Boolean(errors.alias)}
                />
                {errors.alias && <p className='mt-1.5 text-xs text-fk-danger'>{errors.alias}</p>}
              </FieldRow>

              <FieldRow
                label={t('create.display_name.label')}
                description={t('create.display_name.description')}
                htmlFor='create-provider-display-name'
              >
                <Input
                  id='create-provider-display-name'
                  value={values.displayName}
                  onChange={(e) => onChange({ displayName: e.target.value })}
                  placeholder={providerLabel}
                  className='max-w-sm'
                  aria-invalid={Boolean(errors.displayName)}
                />
                {errors.displayName && (
                  <p className='mt-1.5 text-xs text-fk-danger'>{errors.displayName}</p>
                )}
              </FieldRow>

              <FieldRow
                label={t('create.client_id.label')}
                description={t('create.client_id.description', { provider: providerLabel })}
                htmlFor='create-provider-client-id'
              >
                <Input
                  id='create-provider-client-id'
                  value={values.clientId}
                  onChange={(e) => onChange({ clientId: e.target.value })}
                  className='max-w-sm'
                  aria-invalid={Boolean(errors.clientId)}
                />
                {errors.clientId && (
                  <p className='mt-1.5 text-xs text-fk-danger'>{errors.clientId}</p>
                )}
              </FieldRow>

              <FieldRow
                label={t('create.client_secret.label')}
                description={t('create.client_secret.description')}
                htmlFor='create-provider-client-secret'
              >
                <InputGroup className='max-w-sm'>
                  <InputGroupInput
                    id='create-provider-client-secret'
                    type={showSecret ? 'text' : 'password'}
                    value={values.clientSecret}
                    onChange={(e) => onChange({ clientSecret: e.target.value })}
                    className='font-mono-ui'
                    aria-invalid={Boolean(errors.clientSecret)}
                  />
                  <InputGroupAddon align='inline-end'>
                    <InputGroupButton
                      size='icon-xs'
                      aria-label={
                        showSecret ? t('create.client_secret.hide') : t('create.client_secret.show')
                      }
                      onClick={() => setShowSecret((value) => !value)}
                    >
                      {showSecret ? <EyeOff /> : <Eye />}
                    </InputGroupButton>
                  </InputGroupAddon>
                </InputGroup>
                {errors.clientSecret && (
                  <p className='mt-1.5 text-xs text-fk-danger'>{errors.clientSecret}</p>
                )}
              </FieldRow>
            </Section>

            <Section
              title={t('create.endpoints.title')}
              description={
                isCustom
                  ? t('create.endpoints.description_custom')
                  : t('create.endpoints.description_template', { provider: providerLabel })
              }
            >
              <FieldRow
                label={t('create.authorization_url.label')}
                description={t('create.authorization_url.description')}
                htmlFor='create-provider-authorization-url'
              >
                <Input
                  id='create-provider-authorization-url'
                  value={values.authorizationUrl}
                  onChange={(e) => onChange({ authorizationUrl: e.target.value })}
                  placeholder={AUTHORIZATION_URL_PLACEHOLDER}
                  className='max-w-lg'
                  aria-invalid={Boolean(errors.authorizationUrl)}
                />
                {errors.authorizationUrl && (
                  <p className='mt-1.5 text-xs text-fk-danger'>{errors.authorizationUrl}</p>
                )}
              </FieldRow>

              <FieldRow
                label={t('create.token_url.label')}
                description={t('create.token_url.description')}
                htmlFor='create-provider-token-url'
              >
                <Input
                  id='create-provider-token-url'
                  value={values.tokenUrl}
                  onChange={(e) => onChange({ tokenUrl: e.target.value })}
                  placeholder={TOKEN_URL_PLACEHOLDER}
                  className='max-w-lg'
                  aria-invalid={Boolean(errors.tokenUrl)}
                />
                {errors.tokenUrl && (
                  <p className='mt-1.5 text-xs text-fk-danger'>{errors.tokenUrl}</p>
                )}
              </FieldRow>

              <FieldRow
                label={t('create.userinfo_url.label')}
                description={
                  protocol === 'oidc'
                    ? t('create.userinfo_url.description_oidc')
                    : t('create.userinfo_url.description_oauth2')
                }
                htmlFor='create-provider-userinfo-url'
              >
                <Input
                  id='create-provider-userinfo-url'
                  value={values.userinfoUrl}
                  onChange={(e) => onChange({ userinfoUrl: e.target.value })}
                  placeholder={USERINFO_URL_PLACEHOLDER}
                  className='max-w-lg'
                  aria-invalid={Boolean(errors.userinfoUrl)}
                />
                {errors.userinfoUrl && (
                  <p className='mt-1.5 text-xs text-fk-danger'>{errors.userinfoUrl}</p>
                )}
              </FieldRow>

              <FieldRow
                label={t('create.scopes.label')}
                description={t('create.scopes.description')}
              >
                <ChipInput
                  values={values.scopes}
                  onChange={(scopes) => onChange({ scopes })}
                  placeholder={SCOPE_PLACEHOLDER}
                  emptyHint={t('create.scopes.empty_hint')}
                />
              </FieldRow>
            </Section>
          </div>

          <ProviderSetupRail template={template} callbackUrl={callbackUrl} />
        </div>
      )}

      {step === 3 && template && (
        <Section title={t('create.review.title')} description={t('create.review.description')}>
          {reviewRows.map(({ key, value }) => (
            <div
              key={key}
              className='grid gap-x-8 gap-y-1 py-2.5 md:grid-cols-[minmax(0,14rem)_minmax(0,1fr)]'
            >
              <p className='text-xs text-neutral-500 dark:text-neutral-400'>{t(key)}</p>
              <div className='min-w-0 break-all font-mono-ui text-xs text-neutral-900 dark:text-neutral-100'>
                {value}
              </div>
            </div>
          ))}
        </Section>
      )}

      <div className='mt-4 flex items-center justify-between'>
        <Button variant='outline' onClick={onBack}>
          <ArrowLeft /> {step === 1 ? t('create.actions.cancel') : t('create.actions.back')}
        </Button>

        {step < 3 ? (
          <Button onClick={onNext} disabled={!canContinue}>
            {t('create.actions.continue')} <ArrowRight />
          </Button>
        ) : (
          <Button onClick={onSubmit} disabled={isPending}>
            <Plus /> {t('create.actions.submit')}
          </Button>
        )}
      </div>
    </PageShell>
  )
}
