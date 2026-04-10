import AddMemberModalFeature from '../feature/modals/add-member-modal-feature'
import { DataTable } from '@/components/ui/data-table'
import { columns } from '../columns/list-organization-members.column'
import { Trash2 } from 'lucide-react'
import { Schemas } from '@/api/api.client'
import User = Schemas.User

interface PageOrganizationMembersProps {
  members: User[]
  isLoading: boolean
  isError: boolean
  handleRemove: (userId: string) => void
}

export default function PageOrganizationMembers({
  members,
  isLoading,
  handleRemove,
}: PageOrganizationMembersProps) {
  return (
    <div>
      <DataTable
        columns={columns}
        data={members}
        isLoading={isLoading}
        emptyState={<NoMembers />}
        rowActions={[
          {
            label: 'Remove',
            icon: <Trash2 className='h-4 w-4' />,
            onClick: (user) => handleRemove(user.id),
          },
        ]}
      />
    </div>
  )
}

function NoMembers() {
  return (
    <div className='flex flex-col items-center justify-center gap-4 p-8 text-center'>
      <div className='w-24 h-24'>
        <img src='/icons/cadenas.png' alt='' />
      </div>

      <div className='flex flex-col gap-6'>
        <div className='flex flex-col gap-1 w-2/3 mx-auto'>
          <span className='text-lg'>This organization has no members</span>
          <span className='text-muted-foreground text-sm'>
            Add users to this organization to manage their membership and access.
          </span>
        </div>

        <div>
          <AddMemberModalFeature />
        </div>
      </div>
    </div>
  )
}
