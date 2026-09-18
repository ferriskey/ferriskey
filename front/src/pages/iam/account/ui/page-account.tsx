import { useTranslation } from 'react-i18next'
import { Input } from '@/components/ui/input'
import SaveBar from '@/components/kit/save-bar'
import { DetailHeader, FieldRow, IconTile, PageShell, PageTabs, Pill, Section } from '@/components/kit'
import { UserRound } from 'lucide-react'
import { useLocation, useParams } from 'react-router'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'
import { ACCOUNT_URL, RouterParams } from '@/routes/router'
import { Schemas } from '@/api/api.client'

import User = Schemas.User
import { formatDate } from '@/utils/format-date'

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
  const { t } = useTranslation('account')
  const { realm_name } = useParams<RouterParams>()
  const { pathname } = useLocation()

  const base = ACCOUNT_URL(realm_name)
  const tabs = [
    { key: 'overview', label: t('tabs.overview'), href: base },
    { key: 'sessions', label: t('tabs.sessions'), href: `${base}/sessions` },
  ]
  const tab = pathname.endsWith('/sessions') ? 'sessions' : 'overview'

  if (isLoading) {
    return (
      <PageShell>
        <div className='flex items-center gap-3'>
          <div className='size-15 animate-pulse rounded-md bg-neutral-100 dark:bg-fk-raised' />
          <div className='space-y-2'>
            <div className='h-5 w-48 animate-pulse rounded bg-neutral-100 dark:bg-fk-raised' />
            <div className='h-4 w-32 animate-pulse rounded bg-neutral-100 dark:bg-fk-raised' />
          </div>
        </div>
      </PageShell>
    )
  }

  if (!profile) {
    return (
      <PageShell>
        <div className={cn(tokens.surface.panel, 'grid place-items-center px-6 py-16')}>
          <p className='text-sm font-medium text-neutral-700 dark:text-neutral-300'>
            {t('unavailable.title')}
          </p>
          <p className='mt-1 max-w-sm text-center text-sm text-neutral-500 dark:text-neutral-400'>
            {t('unavailable.description')}
          </p>
        </div>
      </PageShell>
    )
  }

  return (
    <PageShell>
      <DetailHeader
        icon={
          <IconTile tone='primary' className='size-15'>
            <UserRound className='size-6' strokeWidth={1.75} />
          </IconTile>
        }
        title={profile.username}
        pills={
          <>
            <Pill tone={profile.email_verified ? 'info' : 'amber'} mono>
              {profile.email_verified ? t('header.email_verified') : t('header.email_unverified')}
            </Pill>
            {!usernameEditable && <Pill tone='neutral'>{t('header.username_locked')}</Pill>}
          </>
        }
        meta={
          <dl className='shrink-0 text-right text-xs text-neutral-500 dark:text-neutral-400'>
            <dt className='sr-only'>{t('header.member_since_label')}</dt>
            <dd className='tnum'>
              {t('header.member_since', { date: formatDate(profile.created_at) })}
            </dd>
            <dt className='sr-only'>{t('header.identifier_label')}</dt>
            <dd className='font-mono-ui text-[11px] text-neutral-400 dark:text-neutral-500'>{profile.id}</dd>
          </dl>
        }
      />

      <PageTabs tabs={tabs} value={tab} className='mt-5' />

      <div className={cn('mt-5', tokens.page.blockGap)}>
        <Section title={t('profile.title')} description={t('profile.description')}>
          <FieldRow
            label={t('profile.username.label')}
            description={
              usernameEditable
                ? t('profile.username.description')
                : t('profile.username.locked_description')
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
            label={t('profile.email.label')}
            description={t('profile.email.description')}
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

          <FieldRow label={t('profile.name.label')} description={t('profile.name.description')}>
            <div className='flex max-w-sm gap-2'>
              <Input
                value={firstname}
                onChange={(e) => onFirstnameChange(e.target.value)}
                placeholder={t('profile.name.firstname_placeholder')}
                aria-label={t('profile.name.firstname_placeholder')}
              />
              <Input
                value={lastname}
                onChange={(e) => onLastnameChange(e.target.value)}
                placeholder={t('profile.name.lastname_placeholder')}
                aria-label={t('profile.name.lastname_placeholder')}
              />
            </div>
          </FieldRow>
        </Section>
      </div>

      <SaveBar
        show={dirtyCount > 0}
        title={t('profile.save_bar.title', { count: dirtyCount })}
        description={t('profile.save_bar.description')}
        onCancel={onDiscard}
        cancelLabel={t('profile.save_bar.cancel')}
        actions={[{ label: t('profile.save_bar.submit'), onClick: onSave }]}
      />
    </PageShell>
  )
}
