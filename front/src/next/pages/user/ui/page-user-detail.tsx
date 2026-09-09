import type { ReactNode } from 'react'
import { ArrowLeft, Bot } from 'lucide-react'
import { Button } from '@/components/kit/button'
import SaveBar from '@/components/kit/save-bar'
import { IconTile, PageTabs, Pill, Squircle, StatusDot, type TabItem } from '@/components/kit'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'
import { isServiceAccount } from '@/utils'
import { Schemas } from '@/api/api.client'
import UserOverviewTab from './user-overview-tab'

import User = Schemas.User
import RequiredAction = Schemas.RequiredAction
import { formatDate } from '@/next/shared/format-date'

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
  const container = cn('mx-auto', tokens.page.maxWidth, tokens.page.padding)

  if (isLoading) {
    return (
      <div className={container}>
        <div className='h-4 w-24 animate-pulse rounded bg-neutral-100 dark:bg-neutral-800' />
        <div className='mt-4 flex items-center gap-3'>
          <div className='size-15 animate-pulse rounded-md bg-neutral-100 dark:bg-neutral-800' />
          <div className='space-y-2'>
            <div className='h-5 w-48 animate-pulse rounded bg-neutral-100 dark:bg-neutral-800' />
            <div className='h-4 w-32 animate-pulse rounded bg-neutral-100 dark:bg-neutral-800' />
          </div>
        </div>
      </div>
    )
  }

  if (!user) {
    return (
      <div className={container}>
        <Button variant='ghost' size='sm' className='-ml-2 mb-2 text-neutral-500 dark:text-neutral-400' onClick={onBack}>
          <ArrowLeft className='size-3.5' />
          Users
        </Button>
        <div className={cn(tokens.surface.panel, 'grid place-items-center px-6 py-16')}>
          <p className='text-sm font-medium text-neutral-700 dark:text-neutral-300'>User not found</p>
          <p className='mt-1 max-w-sm text-center text-sm text-neutral-500 dark:text-neutral-400'>
            It may have been deleted, or it belongs to another realm.
          </p>
        </div>
      </div>
    )
  }

  const serviceAccount = isServiceAccount(user)
  const fullName = [user.firstname, user.lastname].filter(Boolean).join(' ')

  return (
    <div className={container}>
      <Button variant='ghost' size='sm' className='-ml-2 mb-2 text-neutral-500 dark:text-neutral-400' onClick={onBack}>
        <ArrowLeft className='size-3.5' />
        Users
      </Button>

      <div className='flex flex-wrap items-start justify-between gap-4'>
        <div className='flex items-center gap-3'>
          {serviceAccount ? (
            <IconTile tone='violet' className='size-15'>
              <Bot className='size-6' strokeWidth={1.75} />
            </IconTile>
          ) : (
            <Squircle name={user.username} size='xl' />
          )}
          <div className='min-w-0'>
            <h1 className={tokens.header.title}>{user.username}</h1>
            <div className='mt-1.5 flex flex-wrap items-center gap-2'>
              <Pill tone={user.enabled ? 'success' : 'neutral'}>
                <StatusDot on={user.enabled} />
                {user.enabled ? 'enabled' : 'disabled'}
              </Pill>
              <Pill tone={user.email_verified ? 'info' : 'amber'} mono>
                {user.email_verified ? 'email verified' : 'email unverified'}
              </Pill>
              {serviceAccount && (
                <Pill tone='violet' mono>
                  service account
                </Pill>
              )}
            </div>
          </div>
        </div>

        <dl className='shrink-0 text-right text-xs text-neutral-500 dark:text-neutral-400'>
          <dt className='sr-only'>Identity</dt>
          <dd>{fullName || (user.email ?? 'no name recorded')}</dd>
          <dt className='sr-only'>Created at</dt>
          <dd className='tnum'>
            Created{' '}
            {formatDate(user.created_at)}
          </dd>
          <dt className='sr-only'>Identifier</dt>
          <dd className='font-mono-ui text-[11px] text-neutral-400 dark:text-neutral-500'>{user.id}</dd>
        </dl>
      </div>

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
        title={`${dirtyCount} unsaved change${dirtyCount > 1 ? 's' : ''}`}
        description='Review the account before applying the changes.'
        onCancel={onDiscard}
        cancelLabel='Discard'
        actions={[{ label: 'Save changes', onClick: onSave }]}
      />
    </div>
  )
}
