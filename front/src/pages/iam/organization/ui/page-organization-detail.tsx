import type { ReactNode } from 'react'
import { ArrowLeft, Building2 } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/kit/button'
import SaveBar from '@/components/kit/save-bar'
import { DetailHeader, IconTile, PageShell, PageTabs, Pill, StatusDot, type TabItem } from '@/components/kit'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'
import { Schemas } from '@/api/api.client'
import OrganizationSettingsTab, { type OrganizationDraft } from './organization-settings-tab'
import OrganizationAttributesTab from './organization-attributes-tab'
import OrganizationMembersTab from './organization-members-tab'

import Organization = Schemas.Organization
import OrganizationAttribute = Schemas.OrganizationAttribute
import User = Schemas.User
import { formatDate } from '@/utils/format-date'

export interface PageOrganizationDetailProps {
  organization?: Organization
  isLoading: boolean
  tab: string
  tabs: TabItem[]
  draft: OrganizationDraft
  errors: { name?: string; alias?: string }
  isDirty: boolean
  onDraftChange: (patch: Partial<OrganizationDraft>) => void
  onDiscard: () => void
  onSave: () => void
  onDelete: () => void
  attributes: OrganizationAttribute[]
  isLoadingAttributes: boolean
  onUpsertAttribute: (key: string, value: string) => void
  onDeleteAttribute: (key: string) => void
  members: User[]
  availableUsers: User[]
  isLoadingMembers: boolean
  onAddMembers: (userIds: string[]) => void
  onRemoveMember: (user: User) => void
  onManageRoles: (user: User) => void
  groups: ReactNode
  onBack: () => void
}

export default function PageOrganizationDetail({
  organization,
  isLoading,
  tab,
  tabs,
  draft,
  errors,
  isDirty,
  onDraftChange,
  onDiscard,
  onSave,
  onDelete,
  attributes,
  isLoadingAttributes,
  onUpsertAttribute,
  onDeleteAttribute,
  members,
  availableUsers,
  isLoadingMembers,
  onAddMembers,
  onRemoveMember,
  onManageRoles,
  groups,
  onBack,
}: PageOrganizationDetailProps) {
  const { t } = useTranslation('organization')

  const backButton = (
    <Button variant='ghost' size='sm' className='-ml-2 mb-2 text-neutral-500 dark:text-neutral-400' onClick={onBack}>
      <ArrowLeft className='size-3.5' />
      {t('detail.back')}
    </Button>
  )

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

  if (!organization) {
    return (
      <PageShell>
        {backButton}
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

  return (
    <PageShell>
      <DetailHeader
        onBack={onBack}
        backLabel={t('detail.back')}
        icon={
          <IconTile tone='violet' className='size-15'>
            <Building2 className='size-6' strokeWidth={1.75} />
          </IconTile>
        }
        title={organization.name}
        pills={
          <>
            <Pill mono>{organization.alias}</Pill>
            <Pill tone={organization.enabled ? 'success' : 'neutral'}>
              <StatusDot on={organization.enabled} />
              {organization.enabled
                ? t('organization.status.enabled')
                : t('organization.status.disabled')}
            </Pill>
            {organization.domain ? (
              <Pill tone='info' mono>
                {organization.domain}
              </Pill>
            ) : (
              <span className='text-xs text-neutral-400 dark:text-neutral-500'>
                {t('organization.no_domain')}
              </span>
            )}
          </>
        }
        meta={
          <dl className='shrink-0 text-right text-xs text-neutral-500 dark:text-neutral-400'>
            <dt className='sr-only'>{t('detail.meta.created_label')}</dt>
            <dd className='tnum'>
              {t('detail.meta.created', { date: formatDate(organization.created_at) })}
            </dd>
            <dt className='sr-only'>{t('detail.meta.identifier_label')}</dt>
            <dd className='font-mono-ui text-[11px] text-neutral-400 dark:text-neutral-500'>{organization.id}</dd>
          </dl>
        }
      />

      <PageTabs tabs={tabs} value={tab} className='mt-5'>
        <div className={tokens.page.blockGap}>
          {tab === 'settings' && (
            <OrganizationSettingsTab
              organization={organization}
              draft={draft}
              errors={errors}
              onChange={onDraftChange}
              onDelete={onDelete}
            />
          )}

          {tab === 'attributes' && (
            <OrganizationAttributesTab
              attributes={attributes}
              isLoading={isLoadingAttributes}
              onUpsert={onUpsertAttribute}
              onDelete={onDeleteAttribute}
            />
          )}

          {tab === 'members' && (
            <OrganizationMembersTab
              members={members}
              availableUsers={availableUsers}
              isLoading={isLoadingMembers}
              onAdd={onAddMembers}
              onRemove={onRemoveMember}
              onManageRoles={onManageRoles}
            />
          )}

          {tab === 'groups' && groups}
        </div>
      </PageTabs>

      <SaveBar
        show={isDirty}
        title={t('detail.save_bar.title')}
        description={t('detail.save_bar.description')}
        onCancel={onDiscard}
        cancelLabel={t('detail.save_bar.cancel')}
        actions={[{ label: t('detail.save_bar.submit'), onClick: onSave }]}
      />
    </PageShell>
  )
}
