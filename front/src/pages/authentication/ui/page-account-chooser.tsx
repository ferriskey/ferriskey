import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Card, CardContent } from '@/components/ui/card'
import { Skeleton } from '@/components/ui/skeleton'
import { ShieldCheck, UserRound } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { AUTH_NAMESPACE, BRAND_NAME } from '../constants'
import './page-login.css'

export interface ChooserAccount {
  sessionId: string
  displayName: string
  maskedEmail: string | null
  lastUsed: string | null
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

  return (
    <div className='login-shell relative flex min-h-svh items-center justify-center px-6 py-10'>
      <div className='relative z-10 w-full max-w-[380px]'>
        <Card className='login-card overflow-hidden border p-0 shadow-sm'>
          <CardContent className='grid gap-0 p-0'>
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

              {isLoading ? (
                <div
                  className='flex flex-col gap-3'
                  role='status'
                  aria-busy='true'
                  aria-label={t('account_chooser.loading')}
                >
                  <Skeleton className='h-[104px] w-full rounded-lg' />
                  <Skeleton className='h-[104px] w-full rounded-lg' />
                </div>
              ) : (
                <ul className='flex flex-col gap-3' aria-label={t('account_chooser.title')}>
                  {accounts.map((account) => (
                    <li
                      key={account.sessionId}
                      className='flex flex-col gap-3 rounded-lg border bg-card p-3'
                    >
                      <div className='flex items-start gap-3'>
                        <div
                          aria-hidden
                          className='flex h-9 w-9 shrink-0 items-center justify-center rounded-full bg-primary/10 text-primary'
                        >
                          <UserRound className='h-4 w-4' />
                        </div>
                        <div className='min-w-0 flex-1 space-y-0.5'>
                          <p className='truncate text-sm font-medium text-foreground'>
                            {account.displayName}
                          </p>
                          {account.maskedEmail && (
                            <p className='truncate text-xs text-muted-foreground'>
                              {account.maskedEmail}
                            </p>
                          )}
                          {account.lastUsed && (
                            <p className='text-xs text-muted-foreground'>
                              {t('account_chooser.last_used', { when: account.lastUsed })}
                            </p>
                          )}
                        </div>
                      </div>
                      {account.requiresReauthentication && (
                        <Badge
                          variant='outline'
                          className='border-amber-500/30 bg-amber-500/10 text-amber-700 dark:text-amber-400'
                        >
                          <ShieldCheck />
                          {t('account_chooser.reauthentication')}
                        </Badge>
                      )}
                      <div className='flex flex-col gap-1'>
                        <Button
                          type='button'
                          className='h-10 w-full rounded-lg text-sm'
                          disabled={isBusy}
                          aria-busy={busySessionId === account.sessionId}
                          onClick={() => onContinue(account.sessionId)}
                        >
                          {t('account_chooser.continue_as', { name: account.displayName })}
                        </Button>
                        <Button
                          type='button'
                          variant='ghost'
                          className='h-8 w-full rounded-lg text-xs text-muted-foreground'
                          disabled={isBusy}
                          aria-label={t('account_chooser.remove_named', { name: account.displayName })}
                          onClick={() => onRemove(account.sessionId)}
                        >
                          {t('account_chooser.remove')}
                        </Button>
                      </div>
                    </li>
                  ))}
                </ul>
              )}

              <Button
                type='button'
                variant='outline'
                className='h-10 w-full rounded-lg text-sm'
                disabled={isLoading || isBusy}
                onClick={onUseAnother}
              >
                {t('account_chooser.use_another')}
              </Button>
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  )
}
