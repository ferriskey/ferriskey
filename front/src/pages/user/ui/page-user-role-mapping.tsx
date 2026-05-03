import { useMemo } from 'react'
import { OverviewList } from '@/components/ui/overview-list'
import { EntityAvatar } from '@/components/ui/entity-avatar'
import StatisticsCard from '../components/statistics-card'
import RoleMappingModalFeature from '../feature/modals/role-mapping-modal-feature'
import { UserMinus } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Schemas } from '@/api/api.client'
import Role = Schemas.Role

interface PageUserRoleMappingProps {
  userRoles: Role[]
  isLoading: boolean
  isError: boolean
  handleUnassignRole: (roleId: string) => void
  userId?: string
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

export default function PageUserRoleMapping({
  userRoles,
  isLoading,
  handleUnassignRole,
}: PageUserRoleMappingProps) {
  const statistics = useMemo(() => {
    const total = userRoles.length
    const realmRoles = userRoles.filter((r) => !r.client_id).length
    const clientRoles = userRoles.filter((r) => !!r.client_id).length
    const withPermissions = userRoles.filter((r) => r.permissions.length > 0).length
    return { total, realmRoles, clientRoles, withPermissions }
  }, [userRoles])

  return (
    <div className='flex flex-col gap-6'>
      {/* Statistics Cards */}
      <div>
        <p className='text-xs text-muted-foreground mb-3'>Role mapping overview</p>
        <div className='grid gap-4 md:grid-cols-2 lg:grid-cols-4'>
          <StatisticsCard
            title='Assigned roles'
            value={statistics.total}
            description='All roles assigned to this user'
            isLoading={isLoading}
          />
          <StatisticsCard
            title='Realm roles'
            value={statistics.realmRoles}
            description={
              statistics.realmRoles > 0 && statistics.total > 0 ? (
                <span className='text-blue-600 font-medium'>
                  {((statistics.realmRoles / statistics.total) * 100).toFixed(0)}% of total
                </span>
              ) : (
                'No realm roles'
              )
            }
            isLoading={isLoading}
          />
          <StatisticsCard
            title='Client roles'
            value={statistics.clientRoles}
            description='Client-specific roles'
            isLoading={isLoading}
          />
          <StatisticsCard
            title='With permissions'
            value={statistics.withPermissions}
            description='Roles with permissions'
            isLoading={isLoading}
          />
        </div>
      </div>

      {/* Actions */}
      <div className='flex justify-end'>
        <RoleMappingModalFeature />
      </div>

      {/* Role List */}
      <OverviewList
        data={userRoles}
        isLoading={isLoading}
        searchKeys={['name', 'description']}
        searchPlaceholder='Search roles...'
        title={(n) => `Assigned roles (${n})`}
        emptyLabel='No roles assigned to this user.'
        renderRow={(role) => (
          <div className='flex items-center justify-between px-8 py-4 border-b last:border-b-0 hover:bg-muted/40 transition-colors'>
            <div className='flex items-center gap-4'>
              <EntityAvatar label={role.name} color='#6366F1' />
              <div>
                <div className='flex items-center gap-2.5'>
                  <span className='text-base font-medium'>{role.name}</span>
                  <RoleScopeBadge clientId={role.client_id} />
                </div>
                <div className='text-sm text-muted-foreground mt-0.5'>
                  {role.description || (role.client?.client_id ? `client: ${role.client.client_id}` : `role_id: ${role.id}`)}
                </div>
              </div>
            </div>
            <Button
              variant='ghost'
              size='sm'
              className='text-destructive hover:text-destructive hover:bg-destructive/10'
              onClick={() => handleUnassignRole(role.id)}
            >
              <UserMinus className='h-4 w-4' />
            </Button>
          </div>
        )}
      />
    </div>
  )
}
