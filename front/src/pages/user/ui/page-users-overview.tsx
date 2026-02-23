import { Fragment } from 'react/jsx-runtime'
import { Dispatch, SetStateAction, useState, useMemo } from 'react'
import { Schemas } from '@/api/api.client.ts'
import { ConfirmDeleteAlert } from '@/components/confirm-delete-alert'
import { Filter, FilterFieldsConfig } from '@/components/ui/filters'
import StatisticsCard from '../components/statistics-card'
import { OverviewList } from '@/components/ui/overview-list'
import { EntityAvatar } from '@/components/ui/entity-avatar'
import CreateUserModalFeature from '../feature/create-user-modal-feature.tsx'
import { AlertTriangle } from 'lucide-react'
import { isServiceAccount } from '@/utils'

import User = Schemas.User

interface Statistics {
  totalUsers: number
  enabledUsers: number
  disabledUsers: number
  verifiedUsers: number
}

interface ConfirmState {
  title: string
  description: string
  open: boolean
  onConfirm: () => void
}

export interface PageUsersOverviewOverviewProps {
  isLoading?: boolean
  data: User[]
  realmName: string
  statistics: Statistics
  filters: Filter[]
  filterFields: FilterFieldsConfig
  onFiltersChange: (filters: Filter[]) => void
  confirm: ConfirmState
  onConfirmClose: () => void
  handleDeleteSelected: (items: User[]) => void
  handleClickRow: (userId: string) => void
  openCreateUserModal: boolean
  setOpenCreateUserModal: Dispatch<SetStateAction<boolean>>
  onRowDelete: (user: User) => void
}

function UserStatusBadge({ enabled }: { enabled: boolean }) {
  if (!enabled) {
    return (
      <span className='inline-flex items-center gap-1.5 px-3 py-1 rounded-md text-xs font-semibold border border-orange-400/50 text-orange-500 bg-orange-50 dark:bg-orange-500/10'>
        <AlertTriangle className='h-3 w-3' />
        INACTIVE
      </span>
    )
  }
  return (
    <span className='inline-flex items-center px-3 py-1 rounded-md text-xs font-semibold border border-emerald-400/50 text-emerald-600 bg-emerald-50 dark:bg-emerald-500/10'>
      ACTIVE
    </span>
  )
}

function UserTypeBadge({ isSA }: { isSA: boolean }) {
  return isSA ? (
    <span className='inline-flex items-center px-2.5 py-0.5 rounded-md border border-purple-300 text-purple-500 text-xs font-mono bg-purple-50 dark:bg-purple-500/10 dark:border-purple-400/40'>
      service account
    </span>
  ) : (
    <span className='inline-flex items-center px-2.5 py-0.5 rounded-md border border-blue-300 text-blue-500 text-xs font-mono bg-blue-50 dark:bg-blue-500/10 dark:border-blue-400/40'>
      user account
    </span>
  )
}

type TypeFilter = 'all' | 'users' | 'service_accounts'

const typeFilters: { key: TypeFilter; label: string }[] = [
  { key: 'all', label: 'All' },
  { key: 'users', label: 'Users' },
  { key: 'service_accounts', label: 'Service Accounts' },
]

export default function PageUsersOverview({
  isLoading,
  data,
  realmName,
  statistics,
  confirm,
  onConfirmClose,
  handleClickRow,
  openCreateUserModal,
  setOpenCreateUserModal,
}: PageUsersOverviewOverviewProps) {
  const { totalUsers, enabledUsers, disabledUsers, verifiedUsers } = statistics
  const [typeFilter, setTypeFilter] = useState<TypeFilter>('all')

  const filteredData = useMemo(() => {
    if (typeFilter === 'users') return data.filter((u) => !isServiceAccount(u))
    if (typeFilter === 'service_accounts') return data.filter((u) => isServiceAccount(u))
    return data
  }, [data, typeFilter])

  return (
    <Fragment>
      <div className='flex flex-col gap-6'>
        {/* Quick Filters */}
        <div className='flex items-center gap-2'>
          {typeFilters.map((f) => (
            <button
              key={f.key}
              onClick={() => setTypeFilter(f.key)}
              className={`px-4 py-1.5 rounded-md text-sm font-medium transition-colors border ${
                typeFilter === f.key
                  ? 'bg-primary/10 text-primary border-primary/40'
                  : 'bg-transparent text-foreground border-border hover:bg-muted'
              }`}
            >
              {f.label}
            </button>
          ))}
        </div>

        {/* Statistics Cards */}
        <div>
          <p className='text-xs text-muted-foreground mb-3'>User overview</p>
          <div className='grid gap-4 md:grid-cols-2 lg:grid-cols-4'>
            <StatisticsCard
              title='Total users'
              value={totalUsers}
              description='All registered users'
              isLoading={isLoading}
            />
            <StatisticsCard
              title='Enabled users'
              value={enabledUsers}
              description={
                enabledUsers > 0 && totalUsers > 0 ? (
                  <span className='text-emerald-600 font-medium'>
                    {((enabledUsers / totalUsers) * 100).toFixed(0)}% active
                  </span>
                ) : (
                  'No enabled users'
                )
              }
              isLoading={isLoading}
            />
            <StatisticsCard
              title='Disabled users'
              value={disabledUsers}
              description='Inactive accounts'
              isLoading={isLoading}
            />
            <StatisticsCard
              title='Verified users'
              value={verifiedUsers}
              description='Email verified accounts'
              isLoading={isLoading}
            />
          </div>
        </div>

        {/* User List */}
        <OverviewList
          data={filteredData}
          isLoading={isLoading}
          searchKeys={['username', 'email', 'firstname', 'lastname']}
          searchPlaceholder='Search users...'
          title={(n) => `Users (${n})`}
          emptyLabel='No users found.'
          renderRow={(user) => {
            const isSA = isServiceAccount(user)
            const displayName = isSA
              ? 'Service Account'
              : [user.firstname, user.lastname].filter(Boolean).join(' ') || user.username
            return (
              <div
                onClick={() => handleClickRow(user.id)}
                className='flex items-center justify-between px-8 py-4 border-b last:border-b-0 hover:bg-muted/40 cursor-pointer transition-colors'
              >
                <div className='flex items-center gap-4'>
                  <EntityAvatar
                    label={isSA ? 'S' : (user.firstname || user.username || 'U')}
                    color={isSA ? '#8B5CF6' : '#F97316'}
                  />
                  <div>
                    <div className='flex items-center gap-2.5'>
                      <span className='text-base font-medium'>{displayName}</span>
                      <UserTypeBadge isSA={isSA} />
                    </div>
                    <div className='text-sm text-muted-foreground mt-0.5'>
                      {user.email || user.username}
                    </div>
                  </div>
                </div>
                <UserStatusBadge enabled={user.enabled ?? true} />
              </div>
            )
          }}
        />
      </div>

      <CreateUserModalFeature
        realm={realmName}
        open={openCreateUserModal}
        setOpen={setOpenCreateUserModal}
      />
      <ConfirmDeleteAlert
        title={confirm.title}
        description={confirm.description}
        open={confirm.open}
        onConfirm={confirm.onConfirm}
        onCancel={onConfirmClose}
      />
    </Fragment>
  )
}
