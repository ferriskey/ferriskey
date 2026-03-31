import { Schemas } from '@/api/api.client.ts'
import { Button } from '@/components/ui/button'
import { OverviewList } from '@/components/ui/overview-list'
import { Building2, Plus } from 'lucide-react'
import Organization = Schemas.Organization

export interface PageOrganizationsOverviewProps {
  data: Organization[]
  isLoading: boolean
  onRowClick: (organization: Organization) => void
  onCreateClick: () => void
}

export default function PageOrganizationsOverview({
  data,
  isLoading,
  onRowClick,
  onCreateClick,
}: PageOrganizationsOverviewProps) {
  return (
    <div className='flex flex-col gap-6 p-8'>
      <div className='-mx-8 -mt-8 px-8 pt-8 pb-4 border-b flex items-start justify-between'>
        <div>
          <h1 className='text-2xl font-bold tracking-tight'>Organizations</h1>
          <p className='text-sm text-muted-foreground mt-1'>
            Manage the organizations within this realm and their members.
          </p>
        </div>
        <Button onClick={onCreateClick} className='flex items-center gap-2'>
          <Plus className='h-4 w-4' />
          Create organization
        </Button>
      </div>

      <OverviewList
        data={data}
        isLoading={isLoading}
        searchKeys={['name', 'alias']}
        searchPlaceholder='Search organizations...'
        title={(n) => `Organizations (${n})`}
        emptyLabel='No organizations found in this realm.'
        renderRow={(org) => (
          <Button
            variant='ghost'
            className='w-full h-auto px-8 py-4 flex items-center justify-between rounded-none hover:bg-muted/50'
            onClick={() => onRowClick(org)}
          >
            <div className='flex items-center gap-4'>
              <div className='flex h-10 w-10 items-center justify-center rounded-md border bg-muted/50'>
                <Building2 className='h-5 w-5 text-muted-foreground' />
              </div>
              <div className='text-left'>
                <p className='text-sm font-medium'>{org.name}</p>
                <p className='text-xs text-muted-foreground font-mono'>{org.alias}</p>
              </div>
            </div>
            <div className='flex items-center gap-2'>
              {org.domain && (
                <span className='text-xs text-muted-foreground'>{org.domain}</span>
              )}
              <span
                className={`inline-flex items-center px-2 py-0.5 rounded-md border text-xs font-mono ${
                  org.enabled
                    ? 'border-green-300 text-green-600 bg-green-50 dark:bg-green-500/10 dark:border-green-400/40'
                    : 'border-border text-muted-foreground bg-muted/50'
                }`}
              >
                {org.enabled ? 'enabled' : 'disabled'}
              </span>
            </div>
          </Button>
        )}
      />
    </div>
  )
}
