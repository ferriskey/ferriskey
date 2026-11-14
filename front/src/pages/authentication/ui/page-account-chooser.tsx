import { Button } from '@/components/ui/button'
import { Plus, ShieldCheck, X } from 'lucide-react'
import { Trans, useTranslation } from 'react-i18next'
import { AUTH_NAMESPACE } from '../constants'
import AuthShell from '../components/auth-shell'
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

const CONTENT_CLASS = 'p-5 md:p-7'

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

  return (
    <AuthShell
      title={t('account_chooser.title')}
      description={
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
      }
      asideContent={
        errorMessage && (
          <div
            role='alert'
            className='rounded-lg border border-fk-danger-border bg-fk-danger-soft px-3 py-2 text-xs text-fk-danger'
          >
            {errorMessage}
          </div>
        )
      }
      asideFooter={
        !isLoading &&
        accounts.length > 0 && (
          <p className='flex items-center gap-1.5 text-xs text-fk-muted'>
            <ShieldCheck className='size-3.5 text-fk-success' />
            {t('account_chooser.remembered', { count: accounts.length })}
          </p>
        )
      }
      realmName={realmName}
      contentClassName={CONTENT_CLASS}
    >
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
    </AuthShell>
  )
}
