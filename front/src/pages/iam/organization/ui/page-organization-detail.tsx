import type { ReactNode } from 'react'
import { ArrowLeft, Building2 } from 'lucide-react'
import { Button } from '@/components/kit/button'
import SaveBar from '@/components/kit/save-bar'
import { IconTile, PageTabs, Pill, StatusDot, type TabItem } from '@/components/kit'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'
import { Schemas } from '@/api/api.client'
import OrganizationSettingsTab, { type OrganizationDraft } from './organization-settings-tab'
import OrganizationAttributesTab from './organization-attributes-tab'
import OrganizationMembersTab from './organization-members-tab'

import Organization = Schemas.Organization
import OrganizationAttribute = Schemas.OrganizationAttribute
import User = Schemas.User
import { formatDate } from '@/shared/format-date'

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
  const container = cn('mx-auto', tokens.page.maxWidth, tokens.page.padding)

  const backButton = (
    <Button variant='ghost' size='sm' className='-ml-2 mb-2 text-neutral-500 dark:text-neutral-400' onClick={onBack}>
      <ArrowLeft className='size-3.5' />
      Organizations
    </Button>
  )

  if (isLoading) {
    return (
      <div className={container}>
        <div className='h-4 w-24 animate-pulse rounded bg-neutral-100 dark:bg-fk-raised' />
        <div className='mt-4 flex items-center gap-3'>
          <div className='size-15 animate-pulse rounded-md bg-neutral-100 dark:bg-fk-raised' />
          <div className='space-y-2'>
            <div className='h-5 w-48 animate-pulse rounded bg-neutral-100 dark:bg-fk-raised' />
            <div className='h-4 w-32 animate-pulse rounded bg-neutral-100 dark:bg-fk-raised' />
          </div>
        </div>
      </div>
    )
  }

  if (!organization) {
    return (
      <div className={container}>
        {backButton}
        <div className={cn(tokens.surface.panel, 'grid place-items-center px-6 py-16')}>
          <p className='text-sm font-medium text-neutral-700 dark:text-neutral-300'>Organization not found</p>
          <p className='mt-1 max-w-sm text-center text-sm text-neutral-500 dark:text-neutral-400'>
            It may have been deleted, or it belongs to another realm.
          </p>
        </div>
      </div>
    )
  }

  return (
    <div className={container}>
      {backButton}

      <div className='flex flex-wrap items-start justify-between gap-4'>
        <div className='flex items-center gap-3'>
          <IconTile tone='violet' className='size-15'>
            <Building2 className='size-6' strokeWidth={1.75} />
          </IconTile>
          <div className='min-w-0'>
            <h1 className={tokens.header.title}>{organization.name}</h1>
            <div className='mt-1.5 flex flex-wrap items-center gap-2'>
              <Pill mono>{organization.alias}</Pill>
              <Pill tone={organization.enabled ? 'success' : 'neutral'}>
                <StatusDot on={organization.enabled} />
                {organization.enabled ? 'enabled' : 'disabled'}
              </Pill>
              {organization.domain ? (
                <Pill tone='info' mono>
                  {organization.domain}
                </Pill>
              ) : (
                <span className='text-xs text-neutral-400 dark:text-neutral-500'>no domain</span>
              )}
            </div>
          </div>
        </div>

        <dl className='shrink-0 text-right text-xs text-neutral-500 dark:text-neutral-400'>
          <dt className='sr-only'>Created at</dt>
          <dd className='tnum'>
            Created {formatDate(organization.created_at)}
          </dd>
          <dt className='sr-only'>Identifier</dt>
          <dd className='font-mono-ui text-[11px] text-neutral-400 dark:text-neutral-500'>{organization.id}</dd>
        </dl>
      </div>

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
        title='Unsaved changes'
        description='Review the organization before applying the changes.'
        onCancel={onDiscard}
        cancelLabel='Discard'
        actions={[{ label: 'Save changes', onClick: onSave }]}
      />
    </div>
  )
}
