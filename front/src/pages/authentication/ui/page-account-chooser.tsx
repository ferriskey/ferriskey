import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import { Skeleton } from '@/components/ui/skeleton'
import { CircleUser, X } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { AUTH_NAMESPACE, BRAND_NAME } from '../constants'
import { avatarColorClass, avatarInitial } from '../utils/account-avatar'
import './page-login.css'

export interface ChooserAccount {
  sessionId: string
  displayName: string
  maskedEmail: string | null
  requiresReauthentication: boolean
}

export interface PageAccountChooserProps {
  isLoading: boolean
  accounts: ChooserAccount[]
  busySessionId: string | null
  errorMessage: string | null
  onContinue: (sessionId: string) => void
  onRemove: (sessionId: string) => void
  onUseAnother: () => void
}

export default function PageAccountChooser({
  isLoading,
  accounts,
  busySessionId,
  errorMessage,
  onContinue,
  onRemove,
  onUseAnother,
}: PageAccountChooserProps) {
  const { t } = useTranslation(AUTH_NAMESPACE)
  const isBusy = busySessionId !== null

  const rowClass =
    'flex min-h-[52px] w-full items-center gap-3 px-3 py-2.5 text-left transition-colors hover:bg-muted/50 focus-visible:bg-muted/50 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-60'

  return (
    <div className='login-shell relative flex min-h-svh items-center justify-center px-6 py-10'>
      <div className='relative z-10 w-full max-w-[720px]'>
        <Card className='login-card overflow-hidden border p-0 shadow-sm'>
          <CardContent className='grid gap-0 p-0 md:grid-cols-[minmax(0,2fr)_minmax(0,3fr)]'>
            <div className='flex flex-col gap-5 p-6'>
              <div className='space-y-1.5'>
                <div className='flex items-center gap-2'>
                  <img src='/logo_ferriskey.png' alt={BRAND_NAME} className='h-5 w-5 object-contain' />
                  <p className='text-[11px] font-semibold uppercase tracking-[0.2em] text-muted-foreground'>
                    {BRAND_NAME}
                  </p>
                </div>
                <h1 className='login-title text-xl font-semibold tracking-tight text-foreground'>
                  {t('account_chooser.title')}
                </h1>
                <p className='text-sm text-muted-foreground'>{t('account_chooser.description')}</p>
              </div>

              {errorMessage && (
                <div
                  role='alert'
                  className='rounded-md border border-destructive/30 bg-destructive/10 px-3 py-2 text-xs text-destructive'
                >
                  {errorMessage}
                </div>
              )}
            </div>

            <div className='p-6 md:border-l md:self-center'>
              {isLoading ? (
                <div
                  className='divide-y overflow-hidden rounded-lg border bg-card'
                  role='status'
                  aria-busy='true'
                  aria-label={t('account_chooser.loading')}
                >
                  {[0, 1].map((index) => (
                    <div key={index} className='flex min-h-[52px] items-center gap-3 px-3 py-2.5'>
                      <Skeleton className='size-8 shrink-0 rounded-full' />
                      <div className='min-w-0 flex-1 space-y-1.5'>
                        <Skeleton className='h-3.5 w-1/2' />
                        <Skeleton className='h-3 w-3/4' />
                      </div>
                    </div>
                  ))}
                </div>
              ) : (
                <ul
                  className='divide-y overflow-hidden rounded-lg border bg-card'
                  aria-label={t('account_chooser.title')}
                >
                  {accounts.map((account) => (
                    <li key={account.sessionId} className='group relative'>
                      <button
                        type='button'
                        className={`${rowClass} pr-10`}
                        disabled={isBusy}
                        aria-busy={busySessionId === account.sessionId}
                        onClick={() => onContinue(account.sessionId)}
                      >
                        <span
                          aria-hidden
                          className={`flex size-8 shrink-0 items-center justify-center rounded-full text-[10px] font-medium text-white ${avatarColorClass(account.displayName, account.maskedEmail)}`}
                        >
                          {avatarInitial(account.displayName)}
                        </span>
                        <span className='min-w-0 flex-1'>
                          <span className='block truncate text-sm font-medium text-foreground'>
                            {account.displayName}
                          </span>
                          {account.maskedEmail && (
                            <span className='block truncate text-xs text-muted-foreground'>
                              {account.maskedEmail}
                            </span>
                          )}
                        </span>
                        {account.requiresReauthentication && (
                          <span className='shrink-0 self-start text-[11px] text-muted-foreground'>
                            {t('account_chooser.signed_out')}
                          </span>
                        )}
                      </button>
                      <Button
                        type='button'
                        variant='ghost'
                        size='icon'
                        className='absolute right-1.5 top-1/2 size-7 -translate-y-1/2 rounded-md text-muted-foreground opacity-0 group-hover:opacity-100 group-focus-within:opacity-100 focus-visible:opacity-100 [@media(hover:none)]:opacity-100'
                        disabled={isBusy}
                        aria-label={t('account_chooser.remove_named', { name: account.displayName })}
                        title={t('account_chooser.remove')}
                        onClick={() => onRemove(account.sessionId)}
                      >
                        <X className='size-4' />
                      </Button>
                    </li>
                  ))}
                  <li>
                    <button
                      type='button'
                      className={rowClass}
                      disabled={isLoading || isBusy}
                      onClick={onUseAnother}
                    >
                      <span aria-hidden className='flex size-8 shrink-0 items-center justify-center'>
                        <CircleUser className='size-8 text-muted-foreground' strokeWidth={1.5} />
                      </span>
                      <span className='min-w-0 flex-1 truncate text-sm font-medium text-foreground'>
                        {t('account_chooser.use_another')}
                      </span>
                    </button>
                  </li>
                </ul>
              )}
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  )
}
