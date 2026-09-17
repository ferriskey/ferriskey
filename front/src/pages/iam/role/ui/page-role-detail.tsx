import { ArrowLeft, Shield, ShieldCheck } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/kit/button'
import SaveBar from '@/components/kit/save-bar'
import { DetailHeader, IconTile, PageShell, PageTabs, Pill, type TabItem } from '@/components/kit'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'
import { Schemas } from '@/api/api.client'
import RoleSettingsTab from './role-settings-tab'
import RolePermissionsTab from './role-permissions-tab'
import RoleUsersTab from './role-users-tab'

import Role = Schemas.Role
import { formatDate } from '@/utils/format-date'
import { roleScopeLabelKey } from '../role-scope'

export interface PageRoleDetailProps {
  role?: Role
  isLoading: boolean
  tab: string
  tabs: TabItem[]
  name: string
  description: string
  permissions: string[]
  nameError?: string
  dirtyCount: number
  onNameChange: (v: string) => void
  onDescriptionChange: (v: string) => void
  onPermissionsChange: (next: string[]) => void
  onBack: () => void
  onDiscard: () => void
  onSave: () => void
  onDelete: () => void
}

export default function PageRoleDetail({
  role,
  isLoading,
  tab,
  tabs,
  name,
  description,
  permissions,
  nameError,
  dirtyCount,
  onNameChange,
  onDescriptionChange,
  onPermissionsChange,
  onBack,
  onDiscard,
  onSave,
  onDelete,
}: PageRoleDetailProps) {
  const { t } = useTranslation('role')

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

  if (!role) {
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

  const isClientRole = Boolean(role.client_id)

  return (
    <PageShell>
      <DetailHeader
        onBack={onBack}
        backLabel={t('detail.back')}
        icon={
          <IconTile tone={isClientRole ? 'violet' : 'info'} className='size-15'>
            {isClientRole ? (
              <ShieldCheck className='size-6' strokeWidth={1.75} />
            ) : (
              <Shield className='size-6' strokeWidth={1.75} />
            )}
          </IconTile>
        }
        title={role.name}
        pills={
          <>
            <Pill tone={isClientRole ? 'violet' : 'info'} mono>
              {t(roleScopeLabelKey(isClientRole))}
            </Pill>
            <Pill tone={role.permissions.length > 0 ? 'success' : 'amber'}>
              {t('role.permission_count', { count: role.permissions.length })}
            </Pill>
          </>
        }
        meta={
          <dl className='shrink-0 text-right text-xs text-neutral-500 dark:text-neutral-400'>
            <dt className='sr-only'>{t('detail.meta.created_label')}</dt>
            <dd className='tnum'>{t('detail.meta.created', { date: formatDate(role.created_at) })}</dd>
            <dt className='sr-only'>{t('detail.meta.identifier_label')}</dt>
            <dd className='font-mono-ui text-[11px] text-neutral-400 dark:text-neutral-500'>{role.id}</dd>
          </dl>
        }
      />

      <PageTabs tabs={tabs} value={tab} className='mt-5'>
        <div className={tokens.page.blockGap}>
          {tab === 'settings' && (
            <RoleSettingsTab
              role={role}
              name={name}
              description={description}
              nameError={nameError}
              onNameChange={onNameChange}
              onDescriptionChange={onDescriptionChange}
              onDelete={onDelete}
            />
          )}

          {tab === 'permissions' && (
            <RolePermissionsTab value={permissions} onChange={onPermissionsChange} />
          )}

          {tab === 'users' && <RoleUsersTab />}
        </div>
      </PageTabs>

      <SaveBar
        show={dirtyCount > 0}
        title={t('detail.save_bar.title', { count: dirtyCount })}
        description={t('detail.save_bar.description')}
        onCancel={onDiscard}
        cancelLabel={t('detail.save_bar.cancel')}
        actions={[{ label: t('detail.save_bar.submit'), onClick: onSave }]}
      />
    </PageShell>
  )
}
