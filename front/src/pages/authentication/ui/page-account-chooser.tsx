import { useEffect } from 'react'
import { Button } from '@/components/ui/button'
import { KeyRound, Plus, ShieldCheck, X } from 'lucide-react'
import { Trans, useTranslation } from 'react-i18next'
import { AUTH_NAMESPACE, BRAND_NAME } from '../constants'
import { avatarColorClass, avatarInitial } from '../utils/account-avatar'

export interface ChooserAccount {
  sessionId: string
  displayName: string
  maskedEmail: string | null
  requiresReauthentication: boolean
}

export interface PageAccountChooserProps {
  isLoading: boolean
  accounts: ChooserAccount[]
  appName: string
  realmName: string
  busySessionId: string | null
  errorMessage: string | null
  onContinue: (sessionId: string) => void
  onRemove: (sessionId: string) => void
  onUseAnother: () => void
}

const ROW_CLASS =
  'flex w-full items-center gap-3 rounded-xl px-3 py-2.5 text-left transition-colors hover:bg-fk-raised focus-visible:bg-fk-raised focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-fk-brand/40 disabled:pointer-events-none disabled:opacity-60'

export default function PageAccountChooser({
  isLoading,
  accounts,
  appName,
  realmName,
  busySessionId,
  errorMessage,
  onContinue,
  onRemove,
  onUseAnother,
}: PageAccountChooserProps) {
  const { t } = useTranslation(AUTH_NAMESPACE)
  const isBusy = busySessionId !== null

  useEffect(() => {
    document.documentElement.dataset.style = 'ferriskey'
    return () => {
      delete document.documentElement.dataset.style
    }
  }, [])

  return (
    <div className='relative flex min-h-svh items-center justify-center overflow-hidden bg-fk-canvas px-4 py-10 text-fk-ink'>
      <div
        aria-hidden
        className='pointer-events-none absolute inset-0 [background:radial-gradient(55%_45%_at_12%_0%,color-mix(in_oklab,var(--color-fk-brand)_16%,transparent),transparent_70%)]'
      />
      <div
        aria-hidden
        className='pointer-events-none absolute inset-0 opacity-70 [background-image:radial-gradient(var(--color-fk-line)_1px,transparent_1px)] [background-size:22px_22px] [mask-image:radial-gradient(ellipse_at_center,black_25%,transparent_72%)]'
      />

      <div className='relative z-10 w-full max-w-[880px] overflow-hidden rounded-2xl border border-fk-line bg-fk-surface shadow-[0_28px_70px_-30px_rgb(0_0_0/0.35)]'>
        <div className='grid md:grid-cols-[minmax(0,5fr)_minmax(0,6fr)]'>
          <aside className='relative flex flex-col justify-between gap-10 overflow-hidden border-b border-fk-line bg-fk-sand p-7 md:border-r md:border-b-0 md:p-9'>
            <KeyRound
              aria-hidden
              strokeWidth={1}
              className='pointer-events-none absolute -right-6 -bottom-8 size-44 -rotate-[18deg] text-fk-brand/[0.09]'
            />

            <div className='relative space-y-8'>
              <div className='flex items-center gap-2.5'>
                <img src='/logo_ferriskey.png' alt={BRAND_NAME} className='size-5 object-contain' />
                <p className='font-mono text-[11px] font-semibold uppercase tracking-[0.22em] text-fk-primary-text'>
                  {BRAND_NAME}
                </p>
              </div>

              <div className='space-y-3'>
                <h1 className='text-2xl font-semibold tracking-tight text-fk-ink'>
                  {t('account_chooser.title')}
                </h1>
                <p className='text-sm leading-7 text-fk-muted'>
                  <Trans
                    t={t}
                    i18nKey='account_chooser.continue_to'
                    values={{ name: appName }}
                    components={{
                      app: (
                        <span className='inline-flex items-center rounded-md border border-fk-primary-border bg-fk-primary-soft px-1.5 py-0.5 text-[13px] font-medium leading-5 text-fk-primary-text' />
                      ),
                    }}
                  />
                </p>
              </div>

              {errorMessage && (
                <div
                  role='alert'
                  className='rounded-lg border border-fk-danger-border bg-fk-danger-soft px-3 py-2 text-xs text-fk-danger'
                >
                  {errorMessage}
                </div>
              )}
            </div>

            <div className='relative flex flex-wrap items-center gap-x-3 gap-y-2'>
              {!isLoading && accounts.length > 0 && (
                <p className='flex items-center gap-1.5 text-xs text-fk-muted'>
                  <ShieldCheck className='size-3.5 text-fk-success' />
                  {t('account_chooser.remembered', { count: accounts.length })}
                </p>
              )}
              <span className='rounded-md border border-fk-line bg-fk-surface/70 px-1.5 py-0.5 font-mono text-[11px] text-fk-muted'>
                {realmName}
              </span>
            </div>
          </aside>

          <section className='p-5 md:p-7'>
            {isLoading ? (
              <div
                className='space-y-1'
                role='status'
                aria-busy='true'
                aria-label={t('account_chooser.loading')}
              >
                {[0, 1, 2].map((index) => (
                  <div key={index} className='flex items-center gap-3 px-3 py-2.5'>
                    <div className='size-6 shrink-0 animate-pulse rounded-full bg-fk-line' />
                    <div className='min-w-0 flex-1 space-y-1.5'>
                      <div className='h-3.5 w-1/2 animate-pulse rounded bg-fk-line' />
                      <div className='h-3 w-3/4 animate-pulse rounded bg-fk-line-soft' />
                    </div>
                  </div>
                ))}
              </div>
            ) : (
              <ul className='space-y-1' aria-label={t('account_chooser.title')}>
                {accounts.map((account) => (
                  <li key={account.sessionId} className='group relative'>
                    <button
                      type='button'
                      className={`${ROW_CLASS} pr-11`}
                      disabled={isBusy}
                      aria-busy={busySessionId === account.sessionId}
                      onClick={() => onContinue(account.sessionId)}
                    >
                      <span
                        aria-hidden
                        className={`flex size-6 shrink-0 items-center justify-center rounded-full border text-[10px] font-semibold ${avatarColorClass(account.displayName, account.maskedEmail)}`}
                      >
                        {avatarInitial(account.displayName)}
                      </span>
                      <span className='min-w-0 flex-1'>
                        <span className='block truncate text-sm font-medium text-fk-ink'>
                          {account.displayName}
                        </span>
                        {account.maskedEmail && (
                          <span className='block truncate text-xs text-fk-muted'>
                            {account.maskedEmail}
                          </span>
                        )}
                      </span>
                      {account.requiresReauthentication && (
                        <span className='shrink-0 rounded-full border border-fk-amber-border bg-fk-amber-soft px-2 py-0.5 text-[10px] font-medium text-fk-amber'>
                          {t('account_chooser.signed_out')}
                        </span>
                      )}
                    </button>
                    <Button
                      type='button'
                      variant='ghost'
                      size='icon'
                      className='absolute top-1/2 right-2 size-7 -translate-y-1/2 rounded-lg text-fk-muted opacity-0 hover:bg-fk-line hover:text-fk-ink group-hover:opacity-100 group-focus-within:opacity-100 focus-visible:opacity-100 [@media(hover:none)]:opacity-100'
                      disabled={isBusy}
                      aria-label={t('account_chooser.remove_named', { name: account.displayName })}
                      title={t('account_chooser.remove')}
                      onClick={() => onRemove(account.sessionId)}
                    >
                      <X className='size-4' />
                    </Button>
                  </li>
                ))}
                <li className='pt-2'>
                  <button
                    type='button'
                    className='flex w-full items-center gap-3 rounded-xl border border-dashed border-fk-line px-3 py-2.5 text-left transition-colors hover:border-fk-primary-border hover:bg-fk-primary-soft focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-fk-brand/40 disabled:pointer-events-none disabled:opacity-60'
                    disabled={isLoading || isBusy}
                    onClick={onUseAnother}
                  >
                    <span
                      aria-hidden
                      className='flex size-6 shrink-0 items-center justify-center rounded-full border border-dashed border-fk-muted/50 text-fk-muted'
                    >
                      <Plus className='size-3.5' />
                    </span>
                    <span className='min-w-0 flex-1 truncate text-sm font-medium text-fk-ink'>
                      {t('account_chooser.use_another')}
                    </span>
                  </button>
                </li>
              </ul>
            )}
          </section>
        </div>
      </div>
    </div>
  )
}
