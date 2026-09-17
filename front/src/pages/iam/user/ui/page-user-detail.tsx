import type { ReactNode } from 'react'
import { ArrowLeft, Bot } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/kit/button'
import SaveBar from '@/components/kit/save-bar'
import { DetailHeader, IconTile, PageShell, PageTabs, Pill, Squircle, StatusDot, type TabItem } from '@/components/kit'
import { cn } from '@/lib/utils'
import { isServiceAccount } from '@/utils'
import { Schemas } from '@/api/api.client'
import { tokens } from '@/styles/style-tokens'
import UserOverviewTab from './user-overview-tab'

import User = Schemas.User
import RequiredAction = Schemas.RequiredAction
import { formatDate } from '@/utils/format-date'

export interface PageUserDetailProps {
  user?: User
  isLoading: boolean
  tab: string
  tabs: TabItem[]
  username: string
  firstname: string
  lastname: string
  email: string
  enabled: boolean
  emailVerified: boolean
  requiredActions: RequiredAction[]
  emailError?: string
  dirtyCount: number
  onFirstnameChange: (v: string) => void
  onLastnameChange: (v: string) => void
  onEmailChange: (v: string) => void
  onEnabledChange: (v: boolean) => void
  onEmailVerifiedChange: (v: boolean) => void
  onRequiredActionsChange: (next: RequiredAction[]) => void
  onBack: () => void
  onDiscard: () => void
  onSave: () => void
  onDelete: () => void
  children?: ReactNode
}

export default function PageUserDetail({
  user,
  isLoading,
  tab,
  tabs,
  username,
  firstname,
  lastname,
  email,
  enabled,
  emailVerified,
  requiredActions,
  emailError,
  dirtyCount,
  onFirstnameChange,
  onLastnameChange,
  onEmailChange,
  onEnabledChange,
  onEmailVerifiedChange,
  onRequiredActionsChange,
  onBack,
  onDiscard,
  onSave,
  onDelete,
  children,
}: PageUserDetailProps) {
  const { t } = useTranslation('user')

  if (isLoading) {
    return (
      <PageShell>
        <div className='h-4 w-24 animate-pulse rounded bg-neutral-100 dark:bg-fk-raised' />
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

  if (!user) {
    return (
      <PageShell>
        <Button variant='ghost' size='sm' className='-ml-2 mb-2 text-neutral-500 dark:text-neutral-400' onClick={onBack}>
          <ArrowLeft className='size-3.5' />
          {t('detail.back')}
        </Button>
        <div className={cn(tokens.surface.panel, 'grid place-items-center px-6 py-16')}>
          <p className='text-sm font-medium text-neutral-700 dark:text-neutral-300'>
            {t('detail.not_found.title')}
          </p>
          <p className='mt-1 max-w-sm text-center text-sm text-neutral-500 dark:text-neutral-400'>
            {t('detail.not_found.description')}
          </p>
        </div>
      </PageShell>
    )
  }

  const serviceAccount = isServiceAccount(user)
  const fullName = [user.firstname, user.lastname].filter(Boolean).join(' ')

  return (
    <PageShell>
      <DetailHeader
        onBack={onBack}
        backLabel={t('detail.back')}
        icon={
          serviceAccount ? (
            <IconTile tone='violet' className='size-15'>
              <Bot className='size-6' strokeWidth={1.75} />
            </IconTile>
          ) : (
            <Squircle name={user.username} size='xl' />
          )
        }
        title={user.username}
        pills={
          <>
            <Pill tone={user.enabled ? 'success' : 'neutral'}>
              <StatusDot on={user.enabled} />
              {user.enabled ? t('detail.pills.enabled') : t('detail.pills.disabled')}
            </Pill>
            <Pill tone={user.email_verified ? 'info' : 'amber'} mono>
              {user.email_verified
                ? t('detail.pills.email_verified')
                : t('detail.pills.email_unverified')}
            </Pill>
            {serviceAccount && (
              <Pill tone='violet' mono>
                {t('detail.pills.service_account')}
              </Pill>
            )}
          </>
        }
        meta={
          <dl className='shrink-0 grid grid-cols-1 sm:grid-cols-[repeat(auto-fit,minmax(12rem,1fr))] text-right text-xs text-neutral-500 dark:text-neutral-400'>
            <dt className='sr-only'>{t('detail.meta.identity_label')}</dt>
            <dd>{fullName || (user.email ?? t('detail.meta.no_name'))}</dd>
            <dt className='sr-only'>{t('detail.meta.created_label')}</dt>
            <dd className='tnum'>
              {t('detail.meta.created', { date: formatDate(user.created_at) })}
            </dd>
            <dt className='sr-only'>{t('detail.meta.identifier_label')}</dt>
            <dd className='font-mono-ui text-[11px] text-neutral-400 dark:text-neutral-500'>{user.id}</dd>
          </dl>
        }
      />

      <PageTabs tabs={tabs} value={tab} className='mt-5'>
        <div className={tokens.page.blockGap}>
          {tab === 'overview' ? (
            <UserOverviewTab
              user={user}
              username={username}
              firstname={firstname}
              lastname={lastname}
              email={email}
              enabled={enabled}
              emailVerified={emailVerified}
              requiredActions={requiredActions}
              emailError={emailError}
              onFirstnameChange={onFirstnameChange}
              onLastnameChange={onLastnameChange}
              onEmailChange={onEmailChange}
              onEnabledChange={onEnabledChange}
              onEmailVerifiedChange={onEmailVerifiedChange}
              onRequiredActionsChange={onRequiredActionsChange}
              onDelete={onDelete}
            />
          ) : (
            children
          )}
        </div>
      </PageTabs>

      <SaveBar
        show={tab === 'overview' && dirtyCount > 0}
        title={t('detail.save_bar.title', { count: dirtyCount })}
        description={t('detail.save_bar.description')}
        onCancel={onDiscard}
        cancelLabel={t('detail.save_bar.cancel')}
        actions={[{ label: t('detail.save_bar.submit'), onClick: onSave }]}
      />
    </PageShell>
  )
}
