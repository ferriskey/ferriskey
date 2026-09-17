import { Check, Play } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/kit/button'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { Pill, Section } from '@/components/kit'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'
import { Schemas } from '@/api/api.client'

import ClientScope = Schemas.ClientScope
import EvaluateClientScopesResult = Schemas.EvaluateClientScopesResult
import User = Schemas.User

export interface ClientEvaluatePanelProps {
  users: User[]
  optionalScopes: ClientScope[]
  userId: string
  selectedOptional: string[]
  requestedScope: string
  isPending: boolean
  result?: EvaluateClientScopesResult
  onUserChange: (userId: string) => void
  onToggleOptional: (name: string) => void
  onEvaluate: () => void
}

function JsonPanel({ title, value }: { title: string; value: unknown }) {
  return (
    <div>
      <p className='pb-1 text-[11px] font-medium uppercase tracking-wide text-neutral-500 dark:text-neutral-400'>
        {title}
      </p>
      <pre className='max-h-72 overflow-auto rounded-md border border-fk-line bg-neutral-50 px-3 py-2 font-mono-ui text-xs leading-relaxed text-neutral-700 dark:bg-fk-surface dark:text-neutral-300'>
        {JSON.stringify(value ?? null, null, 2)}
      </pre>
    </div>
  )
}

export default function ClientEvaluatePanel({
  users,
  optionalScopes,
  userId,
  selectedOptional,
  requestedScope,
  isPending,
  result,
  onUserChange,
  onToggleOptional,
  onEvaluate,
}: ClientEvaluatePanelProps) {
  const { t } = useTranslation('client')

  return (
    <div className={tokens.page.blockGap}>
      <Section
        title={t('scopes.evaluate.title')}
        description={t('scopes.evaluate.description')}
        contained={false}
      >
        <div className={cn(tokens.surface.panel, 'space-y-4 p-4')}>
          <div className='max-w-sm'>
            <label className='block pb-1 text-xs font-medium text-neutral-900 dark:text-neutral-100' htmlFor='evaluate-user'>
              {t('scopes.evaluate.account')}
            </label>
            <Select value={userId} onValueChange={onUserChange}>
              <SelectTrigger id='evaluate-user' className='w-full'>
                <SelectValue placeholder={t('scopes.evaluate.account_placeholder')} />
              </SelectTrigger>
              <SelectContent>
                {users.map((user) => (
                  <SelectItem key={user.id} value={user.id}>
                    {user.email
                      ? t('scopes.evaluate.account_option', {
                          username: user.username,
                          email: user.email,
                        })
                      : user.username}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          {optionalScopes.length > 0 && (
            <div>
              <p className='pb-1.5 text-xs font-medium text-neutral-900 dark:text-neutral-100'>
                {t('scopes.evaluate.optional')}
              </p>
              <div className='flex flex-wrap gap-1.5'>
                {optionalScopes.map((scope) => {
                  const on = selectedOptional.includes(scope.name)
                  return (
                    <button
                      key={scope.id}
                      type='button'
                      aria-pressed={on}
                      onClick={() => onToggleOptional(scope.name)}
                      className={cn(
                        'inline-flex cursor-pointer items-center gap-1.5 rounded border px-1.5 py-0.5 font-mono-ui text-xs leading-5 transition-colors',
                        on
                          ? 'border-fk-primary-border bg-fk-primary-soft text-fk-primary-text'
                          : 'border-fk-line bg-white text-neutral-500 hover:bg-neutral-50 dark:bg-fk-surface dark:text-neutral-400 dark:hover:bg-fk-surface'
                      )}
                    >
                      {on && <Check className='size-2.5' strokeWidth={3} />}
                      {scope.name}
                    </button>
                  )
                })}
              </div>
            </div>
          )}

          <div>
            <p className='pb-1 text-[11px] font-medium uppercase tracking-wide text-neutral-500 dark:text-neutral-400'>
              {t('scopes.evaluate.scope_string')}
            </p>
            <code className='block rounded-md border border-fk-line bg-neutral-50 px-2.5 py-1.5 font-mono-ui text-xs text-neutral-700 dark:bg-fk-surface dark:text-neutral-300'>
              {requestedScope || t('scopes.evaluate.scope_string_empty')}
            </code>
          </div>

          <Button disabled={!userId || isPending} onClick={onEvaluate}>
            <Play /> {isPending ? t('scopes.evaluate.running') : t('scopes.evaluate.run')}
          </Button>
        </div>
      </Section>

      {result && (
        <>
          <Section
            title={t('scopes.evaluate.mappers.title', { total: result.effective_mappers.length })}
            description={t('scopes.evaluate.mappers.description')}
            contained={result.effective_mappers.length > 0}
          >
            {result.effective_mappers.length > 0 ? (
              <ul className={tokens.surface.divider}>
                {result.effective_mappers.map((mapper, i) => (
                  <li key={`${mapper.name}-${i}`} className='py-2.5'>
                    <p className='text-xs font-medium text-neutral-900 dark:text-neutral-100'>{mapper.name}</p>
                    <p className='font-mono-ui text-[11px] text-neutral-400 dark:text-neutral-500'>
                      {mapper.mapper_type}
                    </p>
                  </li>
                ))}
              </ul>
            ) : (
              <p className='rounded-lg border border-dashed border-fk-line px-4 py-3 text-xs text-neutral-500 dark:text-neutral-400'>
                {t('scopes.evaluate.mappers.empty')}
              </p>
            )}
          </Section>

          <Section
            title={t('scopes.evaluate.roles.title')}
            description={t('scopes.evaluate.roles.description')}
          >
            <div className='space-y-3 py-3'>
              <div>
                <p className='pb-1 text-[11px] uppercase tracking-wide text-neutral-500 dark:text-neutral-400'>
                  {t('scopes.evaluate.roles.realm')}
                </p>
                <div className='flex flex-wrap gap-1.5'>
                  {result.effective_roles.realm_roles.length > 0 ? (
                    result.effective_roles.realm_roles.map((role) => (
                      <Pill key={role} tone='violet' mono>
                        {role}
                      </Pill>
                    ))
                  ) : (
                    <span className='text-xs text-neutral-400 dark:text-neutral-500'>
                      {t('scopes.evaluate.roles.none')}
                    </span>
                  )}
                </div>
              </div>
              {Object.entries(result.effective_roles.client_roles).map(([client, roles]) => (
                <div key={client}>
                  <p className='pb-1 font-mono-ui text-[11px] text-neutral-500 dark:text-neutral-400'>{client}</p>
                  <div className='flex flex-wrap gap-1.5'>
                    {roles.map((role) => (
                      <Pill key={`${client}-${role}`} tone='info' mono>
                        {role}
                      </Pill>
                    ))}
                  </div>
                </div>
              ))}
            </div>
          </Section>

          <Section
            title={t('scopes.evaluate.claims.title')}
            description={t('scopes.evaluate.claims.description')}
            contained={false}
          >
            <div className='space-y-3'>
              <JsonPanel title={t('scopes.evaluate.claims.access_token')} value={result.access_token} />
              <JsonPanel title={t('scopes.evaluate.claims.id_token')} value={result.id_token} />
              <JsonPanel title={t('scopes.evaluate.claims.userinfo')} value={result.userinfo} />
            </div>
          </Section>
        </>
      )}
    </div>
  )
}
