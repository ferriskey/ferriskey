import { useState } from 'react'
import { Shield, Trash2, UserPlus } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/kit/button'
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip'
import {
  DataView,
  EntityPicker,
  IconTile,
  ListingToolbar,
  PaginationBar,
  Pill,
  Section,
  StatusDot,
  type Column,
  type PagedListing,
  type PaginationMetadata,
  type PickableEntity,
  type ViewMode,
} from '@/components/kit'
import { isServiceAccount } from '@/utils'
import { formatRelative } from '@/utils/format-date'
import { Schemas } from '@/api/api.client'
import { memberDisplayName } from '../member-name'

import User = Schemas.User

const SERVICE_ACCOUNT_INITIAL = 'S'
const FALLBACK_INITIAL = 'U'
const EMPTY_VALUE = '—'
const MEMBERS_VIEW: ViewMode = 'list'

export interface OrganizationMemberRow {
  id: string
  joinedAt: string
  user: User
}

export interface OrganizationMembersTabProps {
  rows: OrganizationMemberRow[]
  listing: PagedListing
  pagination?: PaginationMetadata
  isLoading: boolean
  availableUsers: User[]
  onSearchUsers: (search: string) => void
  isSearchingUsers: boolean
  onAdd: (userIds: string[]) => void
  onRemove: (user: User) => void
  onManageRoles: (user: User) => void
}

export default function OrganizationMembersTab({
  rows,
  listing,
  pagination,
  isLoading,
  availableUsers,
  onSearchUsers,
  isSearchingUsers,
  onAdd,
  onRemove,
  onManageRoles,
}: OrganizationMembersTabProps) {
  const { t } = useTranslation('organization')
  const [staged, setStaged] = useState<PickableEntity[]>([])

  const narrowed = Object.values(listing.state.filters).some(Boolean)

  const pickerItems = [
    ...staged,
    ...availableUsers
      .filter((user) => !staged.some((item) => item.id === user.id))
      .map((user) => ({
        id: user.id,
        label: memberDisplayName(user),
        sublabel: user.email ?? user.username,
      })),
  ]

  const avatar = ({ user }: OrganizationMemberRow) => {
    const serviceAccount = isServiceAccount(user)
    return (
      <IconTile tone={serviceAccount ? 'violet' : 'info'}>
        <span className='text-xs font-semibold uppercase'>
          {(serviceAccount
            ? SERVICE_ACCOUNT_INITIAL
            : user.firstname || user.username || FALLBACK_INITIAL
          ).charAt(0)}
        </span>
      </IconTile>
    )
  }

  const kind = ({ user }: OrganizationMemberRow) => {
    const serviceAccount = isServiceAccount(user)
    return (
      <Pill tone={serviceAccount ? 'violet' : 'info'} mono>
        {serviceAccount ? t('member.kind.service_account') : t('member.kind.user_account')}
      </Pill>
    )
  }

  const status = ({ user }: OrganizationMemberRow) => (
    <span className='inline-flex shrink-0 items-center gap-1.5 text-xs text-neutral-600 dark:text-neutral-400'>
      <StatusDot on={user.enabled} />
      {user.enabled ? t('member.status.active') : t('member.status.inactive')}
    </span>
  )

  const actions = ({ user }: OrganizationMemberRow) => (
    <div className='flex items-center justify-end gap-1'>
      <Tooltip>
        <TooltipTrigger asChild>
          <Button
            variant='ghost'
            size='icon'
            className='size-7 text-neutral-400 dark:text-neutral-500'
            aria-label={t('detail.members.manage_roles_for', { name: memberDisplayName(user) })}
            onClick={() => onManageRoles(user)}
          >
            <Shield />
          </Button>
        </TooltipTrigger>
        <TooltipContent>{t('detail.members.manage_roles')}</TooltipContent>
      </Tooltip>
      <Tooltip>
        <TooltipTrigger asChild>
          <Button
            variant='ghost'
            size='icon'
            className='size-7 text-neutral-400 dark:text-neutral-500 hover:text-fk-danger'
            aria-label={t('detail.members.remove_member', { name: memberDisplayName(user) })}
            onClick={() => onRemove(user)}
          >
            <Trash2 />
          </Button>
        </TooltipTrigger>
        <TooltipContent>{t('detail.members.remove')}</TooltipContent>
      </Tooltip>
    </div>
  )

  const columns: Column<OrganizationMemberRow>[] = [
    {
      key: 'member',
      header: t('detail.members.columns.member'),
      render: (row) => (
        <div className='flex items-center gap-3'>
          {avatar(row)}
          <div className='min-w-0'>
            <div className='flex items-center gap-2'>
              <span className='truncate text-[13px] font-medium text-neutral-900 dark:text-neutral-100'>
                {memberDisplayName(row.user)}
              </span>
              {kind(row)}
            </div>
            <p className='truncate text-xs text-neutral-500 dark:text-neutral-400'>
              {row.user.username}
            </p>
          </div>
        </div>
      ),
      sortKey: 'username',
    },
    {
      key: 'email',
      header: t('detail.members.columns.email'),
      render: (row) => (
        <span className='text-sm text-neutral-500 dark:text-neutral-400'>
          {row.user.email ?? EMPTY_VALUE}
        </span>
      ),
      sortKey: 'email',
    },
    {
      key: 'status',
      header: t('detail.members.columns.status'),
      render: status,
      filters: [
        { kind: 'boolean', key: 'enabled', label: t('detail.members.filter_fields.enabled') },
      ],
    },
    {
      key: 'joined',
      header: t('detail.members.columns.joined'),
      render: (row) => <span className='tnum'>{formatRelative(row.joinedAt)}</span>,
      sortKey: 'created_at',
      filters: [
        {
          kind: 'date-range',
          fromKey: 'created_from',
          toKey: 'created_to',
          label: t('detail.members.filter_fields.joined'),
        },
      ],
    },
    {
      key: 'actions',
      header: '',
      align: 'right',
      render: actions,
    },
  ]

  return (
    <>
      <Section
        title={t('detail.members.add.title')}
        description={t('detail.members.add.description')}
        contained={false}
      >
        <div className='space-y-2'>
          <EntityPicker
            items={pickerItems}
            value={staged.map((item) => item.id)}
            onChange={(next) => setStaged(pickerItems.filter((item) => next.includes(item.id)))}
            onSearchChange={onSearchUsers}
            loading={isSearchingUsers}
            addLabel={t('detail.members.add.pick')}
            searchPlaceholder={t('detail.members.add.search_placeholder')}
            emptyHint={t('detail.members.add.empty_hint')}
            exhaustedHint={t('detail.members.add.exhausted_hint')}
          />
          {staged.length > 0 && (
            <Button
              size='sm'
              onClick={() => {
                onAdd(staged.map((item) => item.id))
                setStaged([])
              }}
            >
              <UserPlus /> {t('detail.members.add.submit', { count: staged.length })}
            </Button>
          )}
        </div>
      </Section>

      <Section
        title={t('detail.members.title', { total: pagination?.total ?? 0 })}
        description={t('detail.members.description')}
        contained={false}
      >
        <div className='flex flex-col gap-3'>
          <ListingToolbar
            listing={listing}
            columns={columns}
            search={{ placeholder: t('detail.members.search_placeholder') }}
          />
          <DataView
            rows={rows}
            columns={columns}
            card={{
              avatar,
              title: (row) => memberDisplayName(row.user),
              subtitle: (row) => row.user.email ?? row.user.username,
              badges: (row) => (
                <>
                  {kind(row)}
                  {status(row)}
                </>
              ),
              footer: actions,
            }}
            getKey={(row) => row.id}
            view={MEMBERS_VIEW}
            loading={isLoading}
            listing={listing}
            sort={listing.state.sort}
            onSortChange={listing.setSort}
            emptyLabel={narrowed ? t('detail.members.no_match') : t('detail.members.empty')}
            emptyAction={
              narrowed ? (
                <Button variant='outline' onClick={listing.clearFilters}>
                  {t('detail.members.show_all')}
                </Button>
              ) : undefined
            }
          />
          {pagination && <PaginationBar pagination={pagination} onPageChange={listing.setPage} />}
        </div>
      </Section>
    </>
  )
}
