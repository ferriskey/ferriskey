import { useFormContext } from 'react-hook-form'
import { FormControl, FormField, FormItem, FormLabel } from '@/components/ui/form.tsx'
import { Switch } from '@/components/ui/switch.tsx'
import { UpdateUserSchema } from '../validators'
import { InputText } from '@/components/ui/input-text'
import FloatingActionBar from '@/components/ui/floating-action-bar.tsx'
import { RequiredAction } from '@/api/core.interface'
import MultipleSelector from '@/components/ui/multiselect'
import { Label } from '@/components/ui/label'
import { formatRequiredAction, formatSnakeCaseToTitleCase } from '@/utils'
import { Schemas } from '@/api/api.client'
import { ConfirmDeleteAlert } from '@/components/confirm-delete-alert'
import { Button } from '@/components/ui/button'
import { useState } from 'react'
import User = Schemas.User

type Props = {
  onSubmit: (data: UpdateUserSchema) => void
  hasChanges: boolean
  user: User
  onDelete: () => void
}

export default function PageUserOverview({ onSubmit, hasChanges, user, onDelete }: Props) {
  const form = useFormContext<UpdateUserSchema>()
  const [showDeleteDialog, setShowDeleteDialog] = useState(false)

  const requiredActions = Object.values(RequiredAction).map((action) => ({
    label: formatRequiredAction(action),
    value: action,
  }))

  return (
    <div className='flex flex-col gap-8'>
      {/* User Details */}
      <div className='flex flex-col gap-1'>
        <div className='mb-4'>
          <p className='text-xs text-muted-foreground mb-0.5'>System information</p>
          <h2 className='text-base font-semibold'>User Details</h2>
        </div>

        <div className='flex items-start justify-between py-4 border-t'>
          <div className='w-1/3'>
            <p className='text-sm font-medium'>User ID</p>
            <p className='text-sm text-muted-foreground mt-0.5'>Unique identifier assigned to this user.</p>
          </div>
          <div className='w-1/2'>
            <InputText label='User ID' value={user.id} disabled name='id' />
          </div>
        </div>

        <div className='flex items-start justify-between py-4 border-t'>
          <div className='w-1/3'>
            <p className='text-sm font-medium'>Created At</p>
            <p className='text-sm text-muted-foreground mt-0.5'>Date when this user account was created.</p>
          </div>
          <div className='w-1/2'>
            <InputText
              label='Created At'
              value={new Date(user.created_at).toLocaleDateString('en-US', {
                year: 'numeric',
                month: '2-digit',
                day: '2-digit',
              })}
              disabled
              name='created_at'
            />
          </div>
        </div>

        <FormField
          control={form.control}
          name='required_actions'
          render={({ field }) => (
            <div className='flex items-start justify-between py-4 border-t'>
              <div className='w-1/3'>
                <p className='text-sm font-medium'>Required Actions</p>
                <p className='text-sm text-muted-foreground mt-0.5'>Actions the user must complete on next login.</p>
              </div>
              <div className='w-1/2'>
                <Label className='text-sm text-muted-foreground mb-1.5 block'>Actions</Label>
                <MultipleSelector
                  commandProps={{ label: 'Required Actions' }}
                  onChange={(value) => field.onChange(value.map((v) => v.value))}
                  value={field.value?.map((action) => ({
                    label: formatSnakeCaseToTitleCase(action),
                    value: action,
                  }))}
                  options={requiredActions}
                />
              </div>
            </div>
          )}
        />
      </div>

      {/* General Information */}
      <div className='flex flex-col gap-1'>
        <div className='mb-4'>
          <p className='text-xs text-muted-foreground mb-0.5'>Account configuration</p>
          <h2 className='text-base font-semibold'>General Information</h2>
        </div>

        <FormField
          control={form.control}
          name='username'
          render={({ field }) => (
            <div className='flex items-start justify-between py-4 border-t'>
              <div className='w-1/3'>
                <p className='text-sm font-medium'>Username</p>
                <p className='text-sm text-muted-foreground mt-0.5'>Unique login identifier for this user.</p>
              </div>
              <div className='w-1/2'>
                <InputText label='Username' disabled {...field} />
              </div>
            </div>
          )}
        />

        <FormField
          control={form.control}
          name='email'
          render={({ field }) => (
            <div className='flex items-start justify-between py-4 border-t'>
              <div className='w-1/3'>
                <p className='text-sm font-medium'>Email</p>
                <p className='text-sm text-muted-foreground mt-0.5'>Contact address used for notifications.</p>
              </div>
              <div className='w-1/2'>
                <InputText label='Email' type='email' {...field} />
              </div>
            </div>
          )}
        />

        <FormField
          control={form.control}
          name='firstname'
          render={({ field }) => (
            <div className='flex items-start justify-between py-4 border-t'>
              <div className='w-1/3'>
                <p className='text-sm font-medium'>First Name</p>
              </div>
              <div className='w-1/2'>
                <InputText label='First Name' {...field} />
              </div>
            </div>
          )}
        />

        <FormField
          control={form.control}
          name='lastname'
          render={({ field }) => (
            <div className='flex items-start justify-between py-4 border-t'>
              <div className='w-1/3'>
                <p className='text-sm font-medium'>Last Name</p>
              </div>
              <div className='w-1/2'>
                <InputText label='Last Name' {...field} />
              </div>
            </div>
          )}
        />

        <FormField
          control={form.control}
          name='enabled'
          render={({ field }) => (
            <div className='flex items-center justify-between py-4 border-t'>
              <div className='w-1/3'>
                <p className='text-sm font-medium'>User Enabled</p>
                <p className='text-sm text-muted-foreground mt-0.5'>Disabled users cannot authenticate.</p>
              </div>
              <div className='w-1/2'>
                <FormItem className='flex flex-row items-center gap-3'>
                  <FormControl>
                    <Switch checked={field.value} onCheckedChange={field.onChange} />
                  </FormControl>
                  <FormLabel className='!mt-0 font-normal text-muted-foreground'>
                    {field.value ? 'Enabled' : 'Disabled'}
                  </FormLabel>
                </FormItem>
              </div>
            </div>
          )}
        />

        <FormField
          control={form.control}
          name='email_verified'
          render={({ field }) => (
            <div className='flex items-center justify-between py-4 border-t'>
              <div className='w-1/3'>
                <p className='text-sm font-medium'>Email Verified</p>
                <p className='text-sm text-muted-foreground mt-0.5'>Indicates whether the user's email has been verified.</p>
              </div>
              <div className='w-1/2'>
                <FormItem className='flex flex-row items-center gap-3'>
                  <FormControl>
                    <Switch checked={field.value} onCheckedChange={field.onChange} />
                  </FormControl>
                  <FormLabel className='!mt-0 font-normal text-muted-foreground'>
                    {field.value ? 'Verified' : 'Not verified'}
                  </FormLabel>
                </FormItem>
              </div>
            </div>
          )}
        />
      </div>

      {/* Danger Zone */}
      <div className='flex flex-col gap-1'>
        <div className='mb-4'>
          <p className='text-xs text-destructive/70 mb-0.5'>Irreversible actions</p>
          <h2 className='text-base font-semibold text-destructive'>Danger Zone</h2>
        </div>

        <div className='flex items-center justify-between py-4 border-t border-destructive/20'>
          <div className='w-2/3'>
            <p className='text-sm font-medium'>Delete this user</p>
            <p className='text-sm text-muted-foreground mt-0.5'>
              Once deleted, all associated sessions, credentials, and role assignments will be permanently removed.
            </p>
          </div>
          <Button variant='destructive' onClick={() => setShowDeleteDialog(true)}>
            Delete user
          </Button>
        </div>
      </div>

      <FloatingActionBar
        show={hasChanges}
        title='Save changes'
        actions={[
          {
            label: 'Save',
            variant: 'default',
            onClick: form.handleSubmit(onSubmit),
          },
        ]}
        description="You have unsaved changes. Click 'Save' to apply them."
        onCancel={() => form.reset()}
      />

      <ConfirmDeleteAlert
        open={showDeleteDialog}
        title='Delete user'
        description={`This will permanently delete the user "${user.username}" and all associated data.`}
        onConfirm={onDelete}
        onCancel={() => setShowDeleteDialog(false)}
      />
    </div>
  )
}
