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
import TokenPreviewResult = Schemas.TokenPreviewResult
import User = Schemas.User

export interface ClientEvaluatePanelProps {
  users: User[]
  optionalScopes: ClientScope[]
  userId: string
  selectedOptional: string[]
  requestedScope: string
  isPending: boolean
  result?: TokenPreviewResult
  onUserChange: (userId: string) => void
  onToggleOptional: (name: string) => void
  onEvaluate: () => void
}

const NO_USER = '__none__'

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
            <Select
              value={userId || NO_USER}
              onValueChange={(value) => onUserChange(value === NO_USER ? '' : value)}
            >
              <SelectTrigger id='evaluate-user' className='w-full'>
                <SelectValue placeholder={t('scopes.evaluate.account_placeholder')} />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value={NO_USER}>{t('scopes.evaluate.account_none')}</SelectItem>
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

          <Button disabled={isPending} onClick={onEvaluate}>
            <Play /> {isPending ? t('scopes.evaluate.running') : t('scopes.evaluate.run')}
          </Button>
        </div>
      </Section>

      {result && (
        <>
          <Section
            title={t('scopes.evaluate.active_scopes.title', { total: result.active_scopes.length })}
            description={t('scopes.evaluate.active_scopes.description')}
            contained={false}
          >
            <div className='flex flex-wrap gap-1.5'>
              {result.active_scopes.map((scope) => (
                <Pill key={scope.name} tone={scope.type === 'Optional' ? 'info' : 'violet'} mono>
                  {scope.name}
                </Pill>
              ))}
              {result.active_scopes.length === 0 && (
                <span className='text-xs text-neutral-400 dark:text-neutral-500'>
                  {t('scopes.evaluate.roles.none')}
                </span>
              )}
            </div>
          </Section>

          <Section
            title={t('scopes.evaluate.mappers.title', { total: result.applied_mappers.length })}
            description={t('scopes.evaluate.mappers.description')}
            contained={result.applied_mappers.length > 0}
          >
            {result.applied_mappers.length > 0 ? (
              <ul className={tokens.surface.divider}>
                {result.applied_mappers.map((mapper, i) => (
                  <li key={`${mapper.scope}-${mapper.mapper}-${i}`} className='py-2.5'>
                    <p className='text-xs font-medium text-neutral-900 dark:text-neutral-100'>{mapper.mapper}</p>
                    <p className='font-mono-ui text-[11px] text-neutral-400 dark:text-neutral-500'>
                      {mapper.type} · {mapper.scope}
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
            title={t('scopes.evaluate.claims.title')}
            description={t('scopes.evaluate.claims.description')}
            contained={false}
          >
            <div className='space-y-3'>
              <JsonPanel title={t('scopes.evaluate.claims.access_token')} value={result.access_token_claims} />
              <JsonPanel title={t('scopes.evaluate.claims.id_token')} value={result.id_token_claims} />
              <JsonPanel title={t('scopes.evaluate.claims.userinfo')} value={result.userinfo_claims} />
            </div>
          </Section>
        </>
      )}
    </div>
  )
}
