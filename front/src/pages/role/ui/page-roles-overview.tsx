import { Schemas } from '@/api/api.client'
import { ConfirmDeleteAlert } from '@/components/confirm-delete-alert'
import { Filter, FilterFieldsConfig } from '@/components/ui/filters'
import StatisticsCard from '../components/statistics-card'
import { OverviewList } from '@/components/ui/overview-list'
import { EntityAvatar } from '@/components/ui/entity-avatar'

import Role = Schemas.Role

interface Statistics {
  totalRoles: number
  realmRoles: number
  clientRoles: number
  rolesWithPermissions: number
}

interface ConfirmState {
  title: string
  description: string
  open: boolean
  onConfirm: () => void
}

export interface PageRolesOverviewProps {
  isLoading?: boolean
  data: Role[]
  realmName: string
  statistics: Statistics
  filters: Filter[]
  filterFields: FilterFieldsConfig
  onFiltersChange: (filters: Filter[]) => void
  confirm: ConfirmState
  onConfirmClose: () => void
  handleDeleteSelected: (items: Role[]) => void
  handleClickRow: (roleId: string) => void
  onRowDelete: (role: Role) => void
}

function RoleScopeBadge({ clientId }: { clientId?: string | null }) {
  return clientId ? (
    <span className='inline-flex items-center px-2.5 py-0.5 rounded-md border border-purple-300 text-purple-500 text-xs font-mono bg-purple-50 dark:bg-purple-500/10 dark:border-purple-400/40'>
      client
    </span>
  ) : (
    <span className='inline-flex items-center px-2.5 py-0.5 rounded-md border border-blue-300 text-blue-500 text-xs font-mono bg-blue-50 dark:bg-blue-500/10 dark:border-blue-400/40'>
      realm
    </span>
  )
}

function PermissionsBadge({ count }: { count: number }) {
  return (
    <span className='inline-flex items-center px-3 py-1 rounded-md text-xs font-semibold border border-border text-muted-foreground bg-muted/50'>
      {count} permission{count !== 1 ? 's' : ''}
    </span>
  )
}

export default function PageRolesOverview({
  data,
  isLoading,
  statistics,
  confirm,
  onConfirmClose,
  handleClickRow,
}: PageRolesOverviewProps) {
  const { totalRoles, realmRoles, clientRoles, rolesWithPermissions } = statistics

  return (
    <div className='flex flex-col gap-6'>
      {/* Statistics Cards */}
      <div>
        <p className='text-xs text-muted-foreground mb-3'>Role overview</p>
        <div className='grid gap-4 md:grid-cols-2 lg:grid-cols-4'>
          <StatisticsCard
            title='Total roles'
            value={totalRoles}
            description='All registered roles'
            isLoading={isLoading}
          />
          <StatisticsCard
            title='Realm roles'
            value={realmRoles}
            description={
              realmRoles > 0 && totalRoles > 0 ? (
                <span className='text-blue-600 font-medium'>
                  {((realmRoles / totalRoles) * 100).toFixed(0)}% of total
                </span>
              ) : (
                'No realm roles'
              )
            }
            isLoading={isLoading}
          />
          <StatisticsCard
            title='Client roles'
            value={clientRoles}
            description='Client-specific roles'
            isLoading={isLoading}
          />
          <StatisticsCard
            title='With permissions'
            value={rolesWithPermissions}
            description='Roles with permissions'
            isLoading={isLoading}
          />
        </div>
      </div>

      {/* Role List */}
      <OverviewList
        data={data}
        isLoading={isLoading}
        searchKeys={['name', 'description']}
        searchPlaceholder='Search roles...'
        title={(n) => `Roles (${n})`}
        emptyLabel='No roles found.'
        renderRow={(role) => (
          <div
            onClick={() => handleClickRow(role.id)}
            className='flex items-center justify-between px-8 py-4 border-b last:border-b-0 hover:bg-muted/40 cursor-pointer transition-colors'
          >
            <div className='flex items-center gap-4'>
              <EntityAvatar label={role.name} color='#6366F1' />
              <div>
                <div className='flex items-center gap-2.5'>
                  <span className='text-base font-medium'>{role.name}</span>
                  <RoleScopeBadge clientId={role.client_id} />
                </div>
                <div className='text-sm text-muted-foreground mt-0.5'>
                  {role.description || `role_id: ${role.id}`}
                </div>
              </div>
            </div>
            <PermissionsBadge count={role.permissions.length} />
          </div>
        )}
      />

      <ConfirmDeleteAlert
        title={confirm.title}
        description={confirm.description}
        open={confirm.open}
        onConfirm={confirm.onConfirm}
        onCancel={onConfirmClose}
      />
    </div>
  )
}
