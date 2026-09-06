import { useFormContext } from 'react-hook-form'
import { FormField } from '@/components/ui/form.tsx'
import { InputText } from '@/components/ui/input-text'
import FloatingActionBar from '@/components/ui/floating-action-bar.tsx'
import { UpdateOwnProfileSchema } from '../validators'

type Props = {
  onSubmit: (data: UpdateOwnProfileSchema) => void
  hasChanges: boolean
  usernameEditable: boolean
}

export default function PageAccount({ onSubmit, hasChanges, usernameEditable }: Props) {
  const form = useFormContext<UpdateOwnProfileSchema>()

  return (
    <div className='flex flex-col gap-8'>
      <div className='flex flex-col gap-1'>
        <div className='mb-4'>
          <p className='text-xs text-muted-foreground mb-0.5'>Account</p>
          <h2 className='text-base font-semibold'>Personal information</h2>
        </div>

        <FormField
          control={form.control}
          name='username'
          render={({ field }) => (
            <div className='flex items-start justify-between py-4 border-t'>
              <div className='w-1/3'>
                <p className='text-sm font-medium'>Username</p>
                <p className='text-sm text-muted-foreground mt-0.5'>
                  {usernameEditable
                    ? 'Unique login identifier for your account.'
                    : 'Your administrator has disabled username changes for this realm.'}
                </p>
              </div>
              <div className='w-1/2'>
                <InputText label='Username' disabled={!usernameEditable} {...field} />
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
    </div>
  )
}
