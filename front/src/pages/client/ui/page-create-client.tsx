import { ArrowLeft } from 'lucide-react'
import { UseFormReturn } from 'react-hook-form'
import { CreateClientSchema } from '@/pages/client/schemas/create-client.schema.ts'
import { FormControl, FormField, FormItem, FormLabel } from '@/components/ui/form.tsx'
import { InputText } from '@/components/ui/input-text.tsx'
import { Switch } from '@/components/ui/switch.tsx'
import FloatingActionBar from '@/components/ui/floating-action-bar.tsx'

export interface PageCreateClientProps {
  form: UseFormReturn<CreateClientSchema>
  handleBack: () => void
  handleSubmit: () => void
  formIsValid?: boolean
}

export default function PageCreateClient({ form, handleBack, handleSubmit, formIsValid }: PageCreateClientProps) {
  return (
    <div className='flex flex-col gap-6'>
      {/* Breadcrumb / quick nav */}
      <div className='flex items-center gap-2'>
        <button
          onClick={handleBack}
          className='px-4 py-1.5 rounded-md text-sm font-medium transition-colors border bg-transparent text-foreground border-border hover:bg-muted flex items-center gap-2'
        >
          <ArrowLeft className='h-3.5 w-3.5' />
          Clients
        </button>
        <span className='text-muted-foreground text-sm'>/</span>
        <span className='px-4 py-1.5 rounded-md text-sm font-medium border bg-primary/10 text-primary border-primary/40'>
          New Client
        </span>
      </div>

      {/* Section — same style as list sections in overviews */}
      <div className='-mx-8 border-t border-b overflow-hidden'>
        {/* Section header */}
        <div className='px-8 py-4'>
          <p className='text-xs text-muted-foreground mb-0.5'>Client overview</p>
          <h2 className='text-base font-semibold'>Client Details</h2>
        </div>

        {/* Client ID */}
        <FormField
          control={form.control}
          name='clientId'
          render={({ field }) => (
            <div className='flex items-start justify-between px-8 py-4 border-t'>
              <div className='w-1/3'>
                <p className='text-sm font-medium'>Client ID</p>
                <p className='text-sm text-muted-foreground mt-0.5'>Unique identifier for this client.</p>
              </div>
              <div className='w-1/2'>
                <InputText label='Client ID' {...field} error={form.formState.errors.clientId?.message} />
              </div>
            </div>
          )}
        />

        {/* Name */}
        <FormField
          control={form.control}
          name='name'
          render={({ field }) => (
            <div className='flex items-start justify-between px-8 py-4 border-t'>
              <div className='w-1/3'>
                <p className='text-sm font-medium'>Name</p>
                <p className='text-sm text-muted-foreground mt-0.5'>Display name shown in the UI.</p>
              </div>
              <div className='w-1/2'>
                <InputText label='Name' {...field} error={form.formState.errors.name?.message} />
              </div>
            </div>
          )}
        />

        {/* Enabled */}
        <FormField
          control={form.control}
          name='enabled'
          render={({ field }) => (
            <div className='flex items-center justify-between px-8 py-4 border-t'>
              <div className='w-1/3'>
                <p className='text-sm font-medium'>Client Enabled</p>
                <p className='text-sm text-muted-foreground mt-0.5'>Disabled clients cannot authenticate users.</p>
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

        {/* Client Authentication */}
        <FormField
          control={form.control}
          name='clientAuthentication'
          render={({ field }) => (
            <div className='flex items-center justify-between px-8 py-4 border-t'>
              <div className='w-1/3'>
                <p className='text-sm font-medium'>Client Authentication</p>
                <p className='text-sm text-muted-foreground mt-0.5'>
                  If enabled, clients must authenticate using a secret or certificate.
                </p>
              </div>
              <div className='w-1/2'>
                <FormItem className='flex flex-row items-center gap-3'>
                  <FormControl>
                    <Switch checked={field.value} onCheckedChange={field.onChange} />
                  </FormControl>
                  <FormLabel className='!mt-0 font-normal text-muted-foreground'>
                    {field.value ? 'Confidential' : 'Public'}
                  </FormLabel>
                </FormItem>
              </div>
            </div>
          )}
        />
      </div>

      <FloatingActionBar
        show={formIsValid ?? false}
        title='Create Client'
        onCancel={handleBack}
        description='Create a new client with the specified details.'
        actions={[{ label: 'Create', onClick: handleSubmit }]}
      />
    </div>
  )
}
