import { Input } from '@/components/ui/input'
import SaveBar from '@/components/kit/save-bar'
import { FieldRow, IconTile, Pill, Section } from '@/components/kit'
import { UserRound } from 'lucide-react'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'
import { Schemas } from '@/api/api.client'

import User = Schemas.User
import { formatDate } from '@/shared/format-date'

export interface PageAccountProps {
  profile?: User
  isLoading: boolean
  username: string
  firstname: string
  lastname: string
  email: string
  usernameEditable: boolean
  errors: { username?: string; email?: string }
  dirtyCount: number
  onUsernameChange: (v: string) => void
  onFirstnameChange: (v: string) => void
  onLastnameChange: (v: string) => void
  onEmailChange: (v: string) => void
  onDiscard: () => void
  onSave: () => void
}

export default function PageAccount({
  profile,
  isLoading,
  username,
  firstname,
  lastname,
  email,
  usernameEditable,
  errors,
  dirtyCount,
  onUsernameChange,
  onFirstnameChange,
  onLastnameChange,
  onEmailChange,
  onDiscard,
  onSave,
}: PageAccountProps) {
  const container = cn('mx-auto', tokens.page.maxWidth, tokens.page.padding)

  if (isLoading) {
    return (
      <div className={container}>
        <div className='flex items-center gap-3'>
          <div className='size-15 animate-pulse rounded-md bg-neutral-100 dark:bg-fk-raised' />
          <div className='space-y-2'>
            <div className='h-5 w-48 animate-pulse rounded bg-neutral-100 dark:bg-fk-raised' />
            <div className='h-4 w-32 animate-pulse rounded bg-neutral-100 dark:bg-fk-raised' />
          </div>
        </div>
      </div>
    )
  }

  if (!profile) {
    return (
      <div className={container}>
        <div className={cn(tokens.surface.panel, 'grid place-items-center px-6 py-16')}>
          <p className='text-sm font-medium text-neutral-700 dark:text-neutral-300'>Profile unavailable</p>
          <p className='mt-1 max-w-sm text-center text-sm text-neutral-500 dark:text-neutral-400'>
            Your account could not be loaded for this realm. Sign in again, then retry.
          </p>
        </div>
      </div>
    )
  }

  return (
    <div className={container}>
      <div className='flex flex-wrap items-start justify-between gap-4'>
        <div className='flex items-center gap-3'>
          <IconTile tone='primary' className='size-15'>
            <UserRound className='size-6' strokeWidth={1.75} />
          </IconTile>
          <div className='min-w-0'>
            <h1 className={tokens.header.title}>{profile.username}</h1>
            <div className='mt-1.5 flex flex-wrap items-center gap-2'>
              <Pill tone={profile.email_verified ? 'info' : 'amber'} mono>
                {profile.email_verified ? 'email verified' : 'email unverified'}
              </Pill>
              {!usernameEditable && <Pill tone='neutral'>username locked</Pill>}
            </div>
          </div>
        </div>

        <dl className='shrink-0 text-right text-xs text-neutral-500 dark:text-neutral-400'>
          <dt className='sr-only'>Member since</dt>
          <dd className='tnum'>
            Member since{' '}
            {formatDate(profile.created_at)}
          </dd>
          <dt className='sr-only'>Identifier</dt>
          <dd className='font-mono-ui text-[11px] text-neutral-400 dark:text-neutral-500'>{profile.id}</dd>
        </dl>
      </div>

      <div className={cn('mt-5', tokens.page.blockGap)}>
        <Section
          title='Personal information'
          description='What other members of this realm see, and where your notifications are sent.'
        >
          <FieldRow
            label='Username'
            description={
              usernameEditable
                ? 'Unique login identifier for your account.'
                : 'Your administrator has disabled username changes for this realm.'
            }
            htmlFor='account-username'
          >
            <Input
              id='account-username'
              value={username}
              disabled={!usernameEditable}
              onChange={(e) => onUsernameChange(e.target.value)}
              className='max-w-sm'
              aria-invalid={Boolean(errors.username)}
            />
            {errors.username && (
              <p className='mt-1.5 text-xs text-fk-danger'>{errors.username}</p>
            )}
          </FieldRow>

          <FieldRow
            label='Email'
            description='Contact address used for notifications and password recovery.'
            htmlFor='account-email'
          >
            <Input
              id='account-email'
              type='email'
              value={email}
              onChange={(e) => onEmailChange(e.target.value)}
              className='max-w-sm'
              aria-invalid={Boolean(errors.email)}
            />
            {errors.email && <p className='mt-1.5 text-xs text-fk-danger'>{errors.email}</p>}
          </FieldRow>

          <FieldRow
            label='First and last name'
            description='Optional — shown in the console and in the tokens issued for you.'
          >
            <div className='flex max-w-sm gap-2'>
              <Input
                value={firstname}
                onChange={(e) => onFirstnameChange(e.target.value)}
                placeholder='First name'
                aria-label='First name'
              />
              <Input
                value={lastname}
                onChange={(e) => onLastnameChange(e.target.value)}
                placeholder='Last name'
                aria-label='Last name'
              />
            </div>
          </FieldRow>
        </Section>
      </div>

      <SaveBar
        show={dirtyCount > 0}
        title={`${dirtyCount} unsaved change${dirtyCount > 1 ? 's' : ''}`}
        description='Review your profile before applying the changes.'
        onCancel={onDiscard}
        cancelLabel='Discard'
        actions={[{ label: 'Save changes', onClick: onSave }]}
      />
    </div>
  )
}
