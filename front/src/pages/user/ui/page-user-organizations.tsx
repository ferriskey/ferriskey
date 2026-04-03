import AssignOrganizationModalFeature from '../feature/modals/assign-organization-modal-feature'
import { DataTable } from '@/components/ui/data-table'
import { columns } from '../columns/list-user-organizations.column'
import { Trash2 } from 'lucide-react'
import { Schemas } from '@/api/api.client'
import Organization = Schemas.Organization

interface PageUserOrganizationsProps {
  organizations: Organization[]
  isLoading: boolean
  isError: boolean
  handleRemove: (organizationId: string) => void
}

export default function PageUserOrganizations({
  organizations,
  isLoading,
  handleRemove,
}: PageUserOrganizationsProps) {
  return (
    <div>
      <DataTable
        columns={columns}
        data={organizations}
        isLoading={isLoading}
        emptyState={<NoUserOrganizations />}
        rowActions={[
          {
            label: 'Remove',
            icon: <Trash2 className='h-4 w-4' />,
            onClick: (org) => handleRemove(org.id),
          },
        ]}
      />
    </div>
  )
}

function NoUserOrganizations() {
  return (
    <div className='flex flex-col items-center justify-center gap-4 p-8 text-center'>
      <div className='w-24 h-24'>
        <img src='/icons/cadenas.png' alt='' />
      </div>

      <div className='flex flex-col gap-6'>
        <div className='flex flex-col gap-1 w-2/3 mx-auto'>
          <span className='text-lg'>The user belongs to no organizations</span>
          <span className='text-muted-foreground text-sm'>
            Organizations group users together. Assign this user to an organization to manage their
            membership.
          </span>
        </div>

        <div>
          <AssignOrganizationModalFeature />
        </div>
      </div>
    </div>
  )
}
