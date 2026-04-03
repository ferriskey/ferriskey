import { Button } from '@/components/ui/button'
import { DataTable } from '@/components/ui/data-table'
import { Dialog, DialogContent, DialogTitle, DialogTrigger } from '@/components/ui/dialog'
import { Dispatch, SetStateAction } from 'react'
import { columns } from '../../columns/assign-organization.column'
import { FormField } from '@/components/ui/form'
import { UseFormReturn } from 'react-hook-form'
import { AssignOrganizationSchema } from '../../schemas/assign-organization.schema'
import { Schemas } from '@/api/api.client'
import Organization = Schemas.Organization
import User = Schemas.User

export interface AssignOrganizationModalProps {
  open: boolean
  setOpen: Dispatch<SetStateAction<boolean>>
  organizations: Organization[]
  user: User
  form: UseFormReturn<AssignOrganizationSchema>
  isValid: boolean
  handleSubmit: () => void
}

export default function AssignOrganizationModal({
  open,
  setOpen,
  organizations,
  user,
  form,
  isValid,
  handleSubmit,
}: AssignOrganizationModalProps) {
  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        <Button variant='outline'>Add to organization</Button>
      </DialogTrigger>

      <DialogContent className='!max-w-4xl'>
        <DialogTitle>Assign organizations to {user.username}</DialogTitle>

        <form onSubmit={handleSubmit}>
          <div className='flex flex-col gap-4'>
            <FormField
              control={form.control}
              name='organizationIds'
              render={({ field }) => (
                <DataTable
                  onSelectionChange={(selected) => {
                    const ids = selected.map((org) => org.id)
                    field.onChange(ids)
                  }}
                  emptyState={
                    <div className='flex flex-col items-center justify-center gap-4 p-8 text-center'>
                      <div className='w-32 h-32'>
                        <img src='/icons/cadenas.png' alt='' />
                      </div>
                      <span className='text-lg'>No organization is available</span>
                    </div>
                  }
                  columns={columns}
                  enableSelection
                  data={organizations}
                />
              )}
            />

            <div className='mt-4 flex gap-4'>
              <Button disabled={!isValid}>Assign</Button>
              <Button variant='outline' type='button' onClick={() => setOpen(false)}>
                Cancel
              </Button>
            </div>
          </div>
        </form>
      </DialogContent>
    </Dialog>
  )
}
