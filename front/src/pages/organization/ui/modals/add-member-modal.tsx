import { Button } from '@/components/ui/button'
import { DataTable } from '@/components/ui/data-table'
import { Dialog, DialogContent, DialogTitle, DialogTrigger } from '@/components/ui/dialog'
import { Dispatch, SetStateAction } from 'react'
import { columns } from '../../columns/add-member.column'
import { FormField } from '@/components/ui/form'
import { UseFormReturn } from 'react-hook-form'
import { AddMemberSchema } from '../../schemas/add-member.schema'
import { Schemas } from '@/api/api.client'
import Organization = Schemas.Organization
import User = Schemas.User

export interface AddMemberModalProps {
  open: boolean
  setOpen: Dispatch<SetStateAction<boolean>>
  users: User[]
  organization: Organization
  form: UseFormReturn<AddMemberSchema>
  isValid: boolean
  handleSubmit: () => void
}

export default function AddMemberModal({
  open,
  setOpen,
  users,
  organization,
  form,
  isValid,
  handleSubmit,
}: AddMemberModalProps) {
  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        <Button variant='outline'>Add member</Button>
      </DialogTrigger>

      <DialogContent className='!max-w-4xl'>
        <DialogTitle>Add members to {organization.name}</DialogTitle>

        <form onSubmit={handleSubmit}>
          <div className='flex flex-col gap-4'>
            <FormField
              control={form.control}
              name='userIds'
              render={({ field }) => (
                <DataTable
                  onSelectionChange={(selected) => {
                    field.onChange(selected.map((u) => u.id))
                  }}
                  emptyState={
                    <div className='flex flex-col items-center justify-center gap-4 p-8 text-center'>
                      <div className='w-32 h-32'>
                        <img src='/icons/cadenas.png' alt='' />
                      </div>
                      <span className='text-lg'>No users available to add</span>
                    </div>
                  }
                  columns={columns}
                  enableSelection
                  data={users}
                />
              )}
            />

            <div className='mt-4 flex gap-4'>
              <Button disabled={!isValid}>Add</Button>
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
