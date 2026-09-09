import { useMemo, useState } from 'react'
import { ArrowLeft, ArrowRight, Check, Eye, EyeOff, Plus, Search } from 'lucide-react'
import { Button } from '@/components/kit/button'
import { Input } from '@/components/ui/input'
import {
  InputGroup,
  InputGroupAddon,
  InputGroupButton,
  InputGroupInput,
} from '@/components/ui/input-group'
import { ChipInput, FieldRow, Pill, Section } from '@/components/kit'
import ProviderIcon from '@/pages/identity-providers/components/provider-icon'
import type { ProviderTemplate } from '@/constants/identity-provider-templates'
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

const steps = [
  { n: 1, title: 'Provider' },
  { n: 2, title: 'Configuration' },
  { n: 3, title: 'Review' },
] as const

const categoryLabels: Record<ProviderTemplate['category'], string> = {
  social: 'Social',
  enterprise: 'Enterprise',
  developer: 'Developer tools',
  custom: 'Custom',
}

const categoryOrder: ProviderTemplate['category'][] = [
  'social',
  'enterprise',
  'developer',
  'custom',
]

function Stepper({ current }: { current: number }) {
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
                !done && !active && 'bg-neutral-100 text-neutral-400'
              )}
            >
              {done ? <Check className='size-3' strokeWidth={3} /> : step.n}
            </span>
            <span
              className={cn(
                'text-xs',
                active
                  ? 'font-medium text-neutral-900'
                  : done
                    ? 'text-neutral-600'
                    : 'text-neutral-400'
              )}
            >
              {step.title}
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
  const [search, setSearch] = useState('')
  const [showSecret, setShowSecret] = useState(false)

  const matching = useMemo(() => {
    if (!search) return templates

    const query = search.toLowerCase()
    return templates.filter(
      (t) =>
        t.displayName.toLowerCase().includes(query) ||
        t.description.toLowerCase().includes(query)
    )
  }, [templates, search])

  const groups = useMemo(
    () =>
      categoryOrder
        .map((category) => ({
          category,
          items: matching.filter((t) => t.category === category),
        }))
        .filter((group) => group.items.length > 0),
    [matching]
  )

  const isCustom = template?.id === 'custom'

  return (
    <div className={cn('mx-auto', tokens.page.maxWidth, tokens.page.padding)}>
      <Button variant='ghost' size='sm' className='-ml-2 mb-2 text-neutral-500' onClick={onBack}>
        <ArrowLeft className='size-3.5' />
        Identity Providers
      </Button>

      <div className={tokens.header.spacing}>
        <h1 className={tokens.header.title}>Add an identity provider</h1>
        <p className='mt-0.5 text-sm text-neutral-500'>
          Connecting an external {protocol.toUpperCase()} provider to this realm.
        </p>
      </div>

      <div className={cn(tokens.surface.panel, 'mb-4 px-4 py-3')}>
        <Stepper current={step} />
      </div>

      {step === 1 && (
        <Section
          title='Choose a provider'
          description='A template fills the endpoints and the scopes in; only your application credentials are left.'
          contained={false}
        >
          <div className='space-y-5'>
            <label className='relative flex h-8 max-w-sm items-center'>
              <Search className='pointer-events-none absolute left-2.5 size-3.5 text-neutral-400' />
              <input
                type='search'
                value={search}
                onChange={(e) => setSearch(e.target.value)}
                placeholder='Search providers…'
                className='h-full w-full rounded-md border border-fk-line bg-white pl-8 pr-3 text-sm outline-none placeholder:text-neutral-400 focus:border-fk-primary-border focus:ring-2 focus:ring-fk-primary/15'
              />
            </label>

            {groups.length === 0 && (
              <p className='text-sm text-neutral-500'>
                No provider found matching '{search}'
              </p>
            )}

            {groups.map((group) => (
              <div key={group.category}>
                <p className='pb-2 text-xs font-semibold uppercase tracking-wide text-neutral-500'>
                  {categoryLabels[group.category]}
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
                      <span className='grid size-9 shrink-0 place-items-center rounded-md border border-fk-line bg-white'>
                        <ProviderIcon icon={item.icon} size='sm' />
                      </span>
                      <span className='min-w-0 flex-1'>
                        <span className='flex items-center gap-2'>
                          <span className='truncate text-xs font-medium text-neutral-900'>
                            {item.displayName}
                          </span>
                          <Pill tone={item.provider_type === 'oidc' ? 'violet' : 'amber'} mono>
                            {item.provider_type}
                          </Pill>
                        </span>
                        <span className='mt-0.5 block truncate text-xs text-neutral-500'>
                          {item.description}
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
              title={`Configure ${template.displayName}`}
              description='Credentials issued by the provider console.'
            >
              <FieldRow
                label='Alias'
                description='Technical name of the provider in this realm. It appears in the Redirect URI and cannot be changed after creation.'
                htmlFor='create-provider-alias'
              >
                <Input
                  id='create-provider-alias'
                  value={values.alias}
                  onChange={(e) => onChange({ alias: e.target.value })}
                  placeholder='google-workspace'
                  className='max-w-sm font-mono-ui'
                  aria-invalid={Boolean(errors.alias)}
                />
                {errors.alias && <p className='mt-1.5 text-xs text-fk-danger'>{errors.alias}</p>}
              </FieldRow>

              <FieldRow
                label='Display Name'
                description='Label of the button users click on the login page.'
                htmlFor='create-provider-display-name'
              >
                <Input
                  id='create-provider-display-name'
                  value={values.displayName}
                  onChange={(e) => onChange({ displayName: e.target.value })}
                  placeholder={template.displayName}
                  className='max-w-sm'
                  aria-invalid={Boolean(errors.displayName)}
                />
                {errors.displayName && (
                  <p className='mt-1.5 text-xs text-fk-danger'>{errors.displayName}</p>
                )}
              </FieldRow>

              <FieldRow
                label='Client ID'
                description={`Identifier of the OAuth application created at ${template.displayName}.`}
                htmlFor='create-provider-client-id'
              >
                <Input
                  id='create-provider-client-id'
                  value={values.clientId}
                  onChange={(e) => onChange({ clientId: e.target.value })}
                  className='max-w-sm font-mono-ui'
                  aria-invalid={Boolean(errors.clientId)}
                />
                {errors.clientId && (
                  <p className='mt-1.5 text-xs text-fk-danger'>{errors.clientId}</p>
                )}
              </FieldRow>

              <FieldRow
                label='Client Secret'
                description='Stored encrypted; it is never displayed again after creation.'
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
                      aria-label={showSecret ? 'Hide the secret' : 'Show the secret'}
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
              title='Endpoints'
              description={
                isCustom
                  ? 'Required: no template knows them for a custom provider.'
                  : `Pre-filled from the ${template.displayName} template — change them only for a self-hosted instance.`
              }
            >
              <FieldRow
                label='Authorization URL'
                description='Where the user is sent to grant consent.'
                htmlFor='create-provider-authorization-url'
              >
                <Input
                  id='create-provider-authorization-url'
                  value={values.authorizationUrl}
                  onChange={(e) => onChange({ authorizationUrl: e.target.value })}
                  placeholder='https://provider.com/oauth/authorize'
                  className='max-w-lg font-mono-ui'
                  aria-invalid={Boolean(errors.authorizationUrl)}
                />
                {errors.authorizationUrl && (
                  <p className='mt-1.5 text-xs text-fk-danger'>{errors.authorizationUrl}</p>
                )}
              </FieldRow>

              <FieldRow
                label='Token URL'
                description='Where the authorization code is exchanged for a token.'
                htmlFor='create-provider-token-url'
              >
                <Input
                  id='create-provider-token-url'
                  value={values.tokenUrl}
                  onChange={(e) => onChange({ tokenUrl: e.target.value })}
                  placeholder='https://provider.com/oauth/token'
                  className='max-w-lg font-mono-ui'
                  aria-invalid={Boolean(errors.tokenUrl)}
                />
                {errors.tokenUrl && (
                  <p className='mt-1.5 text-xs text-fk-danger'>{errors.tokenUrl}</p>
                )}
              </FieldRow>

              <FieldRow
                label='Userinfo URL'
                description={
                  protocol === 'oidc'
                    ? 'Optional in OIDC: the claims can come from the ID token.'
                    : 'Where the user profile is read once the token is issued.'
                }
                htmlFor='create-provider-userinfo-url'
              >
                <Input
                  id='create-provider-userinfo-url'
                  value={values.userinfoUrl}
                  onChange={(e) => onChange({ userinfoUrl: e.target.value })}
                  placeholder='https://provider.com/api/userinfo'
                  className='max-w-lg font-mono-ui'
                  aria-invalid={Boolean(errors.userinfoUrl)}
                />
                {errors.userinfoUrl && (
                  <p className='mt-1.5 text-xs text-fk-danger'>{errors.userinfoUrl}</p>
                )}
              </FieldRow>

              <FieldRow label='Scopes' description='Requested at every authorization.'>
                <ChipInput
                  values={values.scopes}
                  onChange={(scopes) => onChange({ scopes })}
                  placeholder='openid'
                  emptyHint='No scope requested — most providers reject an authorization request without one.'
                />
              </FieldRow>
            </Section>
          </div>

          <ProviderSetupRail template={template} callbackUrl={callbackUrl} />
        </div>
      )}

      {step === 3 && template && (
        <Section title='Review' description='What will be created in the realm.'>
          {(
            [
              ['Template', template.displayName],
              ['Protocol', protocol],
              ['Alias', values.alias],
              ['Display Name', values.displayName || template.displayName],
              ['Client ID', values.clientId],
              ['Client Secret', '•'.repeat(Math.min(values.clientSecret.length, 24))],
              ['Redirect URI', callbackUrl],
              ['Authorization URL', values.authorizationUrl],
              ['Token URL', values.tokenUrl],
              ['Userinfo URL', values.userinfoUrl || 'Not set'],
              ['Scopes', values.scopes.join(' ') || 'None'],
            ] as const
          ).map(([label, value]) => (
            <div
              key={label}
              className='grid gap-x-8 gap-y-1 py-2.5 md:grid-cols-[minmax(0,14rem)_minmax(0,1fr)]'
            >
              <p className='text-xs text-neutral-500'>{label}</p>
              <div className='min-w-0 break-all font-mono-ui text-xs text-neutral-900'>
                {value}
              </div>
            </div>
          ))}
        </Section>
      )}

      <div className='mt-4 flex items-center justify-between'>
        <Button variant='outline' onClick={onBack}>
          <ArrowLeft /> {step === 1 ? 'Cancel' : 'Back'}
        </Button>

        {step < 3 ? (
          <Button onClick={onNext} disabled={!canContinue}>
            Continue <ArrowRight />
          </Button>
        ) : (
          <Button onClick={onSubmit} disabled={isPending}>
            <Plus /> Create provider
          </Button>
        )}
      </div>
    </div>
  )
}
