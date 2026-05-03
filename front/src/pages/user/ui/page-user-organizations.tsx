import { useMemo } from 'react'
import { OverviewList } from '@/components/ui/overview-list'
import { EntityAvatar } from '@/components/ui/entity-avatar'
import StatisticsCard from '../components/statistics-card'
import AssignOrganizationModalFeature from '../feature/modals/assign-organization-modal-feature'
import { UserMinus } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Schemas } from '@/api/api.client'
import Organization = Schemas.Organization

interface PageUserOrganizationsProps {
  organizations: Organization[]
  isLoading: boolean
  isError: boolean
  handleRemove: (organizationId: string) => void
}

function OrgStatusBadge({ enabled }: { enabled: boolean }) {
  return enabled ? (
    <span className='inline-flex items-center px-2.5 py-0.5 rounded-md border border-emerald-400/50 text-emerald-600 text-xs font-mono bg-emerald-50 dark:bg-emerald-500/10'>
      enabled
    </span>
  ) : (
    <span className='inline-flex items-center px-2.5 py-0.5 rounded-md border border-border text-muted-foreground text-xs font-mono bg-muted/50'>
      disabled
    </span>
  )
}

export default function PageUserOrganizations({
  organizations,
  isLoading,
  handleRemove,
}: PageUserOrganizationsProps) {
  const statistics = useMemo(() => {
    const total = organizations.length
    const enabled = organizations.filter((o) => o.enabled).length
    const disabled = total - enabled
    const withDomain = organizations.filter((o) => !!o.domain).length
    return { total, enabled, disabled, withDomain }
  }, [organizations])

  return (
    <div className='flex flex-col gap-6'>
      {/* Statistics Cards */}
      <div>
        <p className='text-xs text-muted-foreground mb-3'>Organization membership overview</p>
        <div className='grid gap-4 md:grid-cols-2 lg:grid-cols-4'>
          <StatisticsCard
            title='Organizations'
            value={statistics.total}
            description='All memberships for this user'
            isLoading={isLoading}
          />
          <StatisticsCard
            title='Enabled'
            value={statistics.enabled}
            description={
              statistics.enabled > 0 && statistics.total > 0 ? (
                <span className='text-emerald-600 font-medium'>
                  {((statistics.enabled / statistics.total) * 100).toFixed(0)}% of total
                </span>
              ) : (
                'No enabled organizations'
              )
            }
            isLoading={isLoading}
          />
          <StatisticsCard
            title='Disabled'
            value={statistics.disabled}
            description='Inactive organizations'
            isLoading={isLoading}
          />
          <StatisticsCard
            title='With domain'
            value={statistics.withDomain}
            description='Organizations with a domain'
            isLoading={isLoading}
          />
        </div>
      </div>

      {/* Actions */}
      <div className='flex justify-end'>
        <AssignOrganizationModalFeature />
      </div>

      {/* Organization List */}
      <OverviewList
        data={organizations}
        isLoading={isLoading}
        searchKeys={['name', 'alias', 'domain']}
        searchPlaceholder='Search organizations...'
        title={(n) => `Organizations (${n})`}
        emptyLabel='No organizations assigned to this user.'
        renderRow={(org) => (
          <div className='flex items-center justify-between px-8 py-4 border-b last:border-b-0 hover:bg-muted/40 transition-colors'>
            <div className='flex items-center gap-4'>
              <EntityAvatar label={org.name} color='#F59E0B' />
              <div>
                <div className='flex items-center gap-2.5'>
                  <span className='text-base font-medium'>{org.name}</span>
                  <span className='text-xs font-mono text-muted-foreground'>{org.alias}</span>
                </div>
                <div className='text-sm text-muted-foreground mt-0.5'>
                  {org.domain ?? `org_id: ${org.id}`}
                </div>
              </div>
            </div>
            <div className='flex items-center gap-2'>
              <OrgStatusBadge enabled={org.enabled} />
              <Button
                variant='ghost'
                size='sm'
                className='text-destructive hover:text-destructive hover:bg-destructive/10'
                onClick={() => handleRemove(org.id)}
              >
                <UserMinus className='h-4 w-4' />
              </Button>
            </div>
          </div>
        )}
      />
    </div>
  )
}
