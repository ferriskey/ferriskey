import { ArrowLeft } from 'lucide-react'
import { UseFormReturn } from 'react-hook-form'
import { FormControl, FormField, FormItem, FormLabel } from '@/components/ui/form'
import { InputText } from '@/components/ui/input-text'
import { Switch } from '@/components/ui/switch'
import FloatingActionBar from '@/components/ui/floating-action-bar'
import { CreateUserSchema } from '../validators'

export interface PageCreateUserProps {
  form: UseFormReturn<CreateUserSchema>
  handleBack: () => void
  handleSubmit: () => void
  formIsValid?: boolean
}

export default function PageCreateUser({ form, handleBack, handleSubmit, formIsValid }: PageCreateUserProps) {
  return (
    <div className='flex flex-col gap-6'>
      {/* Breadcrumb / quick nav */}
      <div className='flex items-center gap-2'>
        <button
          onClick={handleBack}
          className='px-4 py-1.5 rounded-md text-sm font-medium transition-colors border bg-transparent text-foreground border-border hover:bg-muted flex items-center gap-2'
        >
          <ArrowLeft className='h-3.5 w-3.5' />
          Users
        </button>
        <span className='text-muted-foreground text-sm'>/</span>
        <span className='px-4 py-1.5 rounded-md text-sm font-medium border bg-primary/10 text-primary border-primary/40'>
          New User
        </span>
      </div>

      {/* Section — same style as list sections in overviews */}
      <div className='-mx-8 border-t border-b overflow-hidden'>
        {/* Section header */}
        <div className='px-8 py-4'>
          <p className='text-xs text-muted-foreground mb-0.5'>User overview</p>
          <h2 className='text-base font-semibold'>User Details</h2>
        </div>

        {/* Username */}
        <FormField
          control={form.control}
          name='username'
          render={({ field }) => (
            <div className='flex items-start justify-between px-8 py-4 border-t'>
              <div className='w-1/3'>
                <p className='text-sm font-medium'>Username</p>
                <p className='text-sm text-muted-foreground mt-0.5'>Unique login name for this user.</p>
              </div>
              <div className='w-1/2'>
                <InputText label='Username' {...field} error={form.formState.errors.username?.message} />
              </div>
            </div>
          )}
        />

        {/* First Name */}
        <FormField
          control={form.control}
          name='firstname'
          render={({ field }) => (
            <div className='flex items-start justify-between px-8 py-4 border-t'>
              <div className='w-1/3'>
                <p className='text-sm font-medium'>First Name</p>
                <p className='text-sm text-muted-foreground mt-0.5'>User's given name.</p>
              </div>
              <div className='w-1/2'>
                <InputText label='First Name' {...field} error={form.formState.errors.firstname?.message} />
              </div>
            </div>
          )}
        />

        {/* Last Name */}
        <FormField
          control={form.control}
          name='lastname'
          render={({ field }) => (
            <div className='flex items-start justify-between px-8 py-4 border-t'>
              <div className='w-1/3'>
                <p className='text-sm font-medium'>Last Name</p>
                <p className='text-sm text-muted-foreground mt-0.5'>User's family name.</p>
              </div>
              <div className='w-1/2'>
                <InputText label='Last Name' {...field} error={form.formState.errors.lastname?.message} />
              </div>
            </div>
          )}
        />

        {/* Email */}
        <FormField
          control={form.control}
          name='email'
          render={({ field }) => (
            <div className='flex items-start justify-between px-8 py-4 border-t'>
              <div className='w-1/3'>
                <p className='text-sm font-medium'>Email</p>
                <p className='text-sm text-muted-foreground mt-0.5'>Contact email address for this user.</p>
              </div>
              <div className='w-1/2'>
                <InputText label='Email' {...field} error={form.formState.errors.email?.message} />
              </div>
            </div>
          )}
        />

        {/* Email Verified */}
        <FormField
          control={form.control}
          name='email_verified'
          render={({ field }) => (
            <div className='flex items-center justify-between px-8 py-4 border-t'>
              <div className='w-1/3'>
                <p className='text-sm font-medium'>Email Verified</p>
                <p className='text-sm text-muted-foreground mt-0.5'>
                  Choose between verified and unverified email as default status.
                </p>
              </div>
              <div className='w-1/2'>
                <FormItem className='flex flex-row items-center gap-3'>
                  <FormControl>
                    <Switch checked={field.value} onCheckedChange={field.onChange} />
                  </FormControl>
                  <FormLabel className='!mt-0 font-normal text-muted-foreground'>
                    {field.value ? 'Verified' : 'Unverified'}
                  </FormLabel>
                </FormItem>
              </div>
            </div>
          )}
        />
      </div>

      <FloatingActionBar
        show={formIsValid ?? false}
        title='Create User'
        onCancel={handleBack}
        description='Create a new user with the specified details.'
        actions={[{ label: 'Create', onClick: handleSubmit }]}
      />
    </div>
  )
}
