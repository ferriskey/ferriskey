import { UseFormReturn } from 'react-hook-form'
import { CreateOrganizationSchema } from '../schemas/create-organization.schema'
import { FormControl, FormField, FormItem, FormLabel } from '@/components/ui/form'
import { InputText } from '@/components/ui/input-text'
import { Switch } from '@/components/ui/switch'
import FloatingActionBar from '@/components/ui/floating-action-bar'
import { ArrowLeft } from 'lucide-react'

export interface PageCreateOrganizationProps {
  form: UseFormReturn<CreateOrganizationSchema>
  handleBack: () => void
  handleSubmit: () => void
  formIsValid?: boolean
}

export default function PageCreateOrganization({
  form,
  handleBack,
  handleSubmit,
  formIsValid,
}: PageCreateOrganizationProps) {
  return (
    <div className='flex flex-col gap-6 p-8'>
      <div className='flex items-center gap-2'>
        <button
          onClick={handleBack}
          className='px-4 py-1.5 rounded-md text-sm font-medium transition-colors border bg-transparent text-foreground border-border hover:bg-muted flex items-center gap-2'
        >
          <ArrowLeft className='h-3.5 w-3.5' />
          Organizations
        </button>
        <span className='text-muted-foreground text-sm'>/</span>
        <span className='px-4 py-1.5 rounded-md text-sm font-medium border bg-primary/10 text-primary border-primary/40'>
          New Organization
        </span>
      </div>

      <div className='-mx-8 border-t border-b overflow-hidden'>
        <div className='px-8 py-4'>
          <p className='text-xs text-muted-foreground mb-0.5'>Organization overview</p>
          <h2 className='text-base font-semibold'>Organization Details</h2>
        </div>

        <FormField
          control={form.control}
          name='name'
          render={({ field }) => (
            <div className='flex items-start justify-between px-8 py-4 border-t'>
              <div className='w-1/3'>
                <p className='text-sm font-medium'>Name</p>
                <p className='text-sm text-muted-foreground mt-0.5'>Display name of the organization.</p>
              </div>
              <div className='w-1/2'>
                <InputText label='Name' {...field} error={form.formState.errors.name?.message} />
              </div>
            </div>
          )}
        />

        <FormField
          control={form.control}
          name='alias'
          render={({ field }) => (
            <div className='flex items-start justify-between px-8 py-4 border-t'>
              <div className='w-1/3'>
                <p className='text-sm font-medium'>Alias</p>
                <p className='text-sm text-muted-foreground mt-0.5'>
                  Unique slug used in URLs. Lowercase letters, numbers, hyphens and underscores only.
                </p>
              </div>
              <div className='w-1/2'>
                <InputText
                  label='Alias'
                  {...field}
                  error={form.formState.errors.alias?.message}
                />
              </div>
            </div>
          )}
        />

        <FormField
          control={form.control}
          name='enabled'
          render={({ field }) => (
            <div className='flex items-center justify-between px-8 py-4 border-t'>
              <div className='w-1/3'>
                <p className='text-sm font-medium'>Enabled</p>
                <p className='text-sm text-muted-foreground mt-0.5'>
                  Disabled organizations are not accessible to members.
                </p>
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
          name='domain'
          render={({ field }) => (
            <div className='flex items-start justify-between px-8 py-4 border-t'>
              <div className='w-1/3'>
                <p className='text-sm font-medium'>Domain</p>
                <p className='text-sm text-muted-foreground mt-0.5'>Optional email domain for this organization.</p>
              </div>
              <div className='w-1/2'>
                <InputText
                  label='Domain'
                  {...field}
                  value={field.value ?? ''}
                  error={form.formState.errors.domain?.message}
                />
              </div>
            </div>
          )}
        />

        <FormField
          control={form.control}
          name='redirectUrl'
          render={({ field }) => (
            <div className='flex items-start justify-between px-8 py-4 border-t'>
              <div className='w-1/3'>
                <p className='text-sm font-medium'>Redirect URL</p>
                <p className='text-sm text-muted-foreground mt-0.5'>Optional post-login redirect URL.</p>
              </div>
              <div className='w-1/2'>
                <InputText
                  label='Redirect URL'
                  {...field}
                  value={field.value ?? ''}
                  error={form.formState.errors.redirectUrl?.message}
                />
              </div>
            </div>
          )}
        />

        <FormField
          control={form.control}
          name='description'
          render={({ field }) => (
            <div className='flex items-start justify-between px-8 py-4 border-t'>
              <div className='w-1/3'>
                <p className='text-sm font-medium'>Description</p>
                <p className='text-sm text-muted-foreground mt-0.5'>Optional description.</p>
              </div>
              <div className='w-1/2'>
                <InputText
                  label='Description'
                  {...field}
                  value={field.value ?? ''}
                  error={form.formState.errors.description?.message}
                />
              </div>
            </div>
          )}
        />
      </div>

      <FloatingActionBar
        show={formIsValid ?? false}
        title='Create Organization'
        onCancel={handleBack}
        description='Create a new organization in this realm.'
        actions={[{ label: 'Create', onClick: handleSubmit }]}
      />
    </div>
  )
}
