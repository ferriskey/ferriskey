import { ArrowLeft } from 'lucide-react'
import { UseFormReturn } from 'react-hook-form'
import {
  FormControl,
  FormField,
  FormItem,
} from '@/components/ui/form'
import { InputText } from '@/components/ui/input-text'
import FloatingActionBar from '@/components/ui/floating-action-bar'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { CreateClientScopeSchema } from '../schemas/create-client-scope.schema'

export interface PageCreateClientScopeProps {
  form: UseFormReturn<CreateClientScopeSchema>
  handleBack: () => void
  handleSubmit: () => void
  formIsValid?: boolean
}

export default function PageCreateClientScope({
  form,
  handleBack,
  handleSubmit,
  formIsValid,
}: PageCreateClientScopeProps) {
  return (
    <div className='flex flex-col gap-6'>
      <div className='flex items-center gap-2'>
        <button
          onClick={handleBack}
          className='px-4 py-1.5 rounded-md text-sm font-medium transition-colors border bg-transparent text-foreground border-border hover:bg-muted flex items-center gap-2'
        >
          <ArrowLeft className='h-3.5 w-3.5' />
          Client Scopes
        </button>
        <span className='text-muted-foreground text-sm'>/</span>
        <span className='px-4 py-1.5 rounded-md text-sm font-medium border bg-primary/10 text-primary border-primary/40'>
          New Client Scope
        </span>
      </div>

      <div className='-mx-8 border-t border-b overflow-hidden'>
        <div className='px-8 py-4'>
          <p className='text-xs text-muted-foreground mb-0.5'>Client scope overview</p>
          <h2 className='text-base font-semibold'>Client Scope Details</h2>
        </div>

        <FormField
          control={form.control}
          name='name'
          render={({ field }) => (
            <div className='flex items-start justify-between px-8 py-4 border-t'>
              <div className='w-1/3'>
                <p className='text-sm font-medium'>Name</p>
                <p className='text-sm text-muted-foreground mt-0.5'>Unique name for this client scope.</p>
              </div>
              <div className='w-1/2'>
                <InputText label='Name' {...field} error={form.formState.errors.name?.message} />
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
                <p className='text-sm text-muted-foreground mt-0.5'>Optional description for this client scope.</p>
              </div>
              <div className='w-1/2'>
                <InputText label='Description' {...field} error={form.formState.errors.description?.message} />
              </div>
            </div>
          )}
        />

        <FormField
          control={form.control}
          name='protocol'
          render={({ field }) => (
            <div className='flex items-start justify-between px-8 py-4 border-t'>
              <div className='w-1/3'>
                <p className='text-sm font-medium'>Protocol</p>
                <p className='text-sm text-muted-foreground mt-0.5'>Only OpenID Connect is currently supported.</p>
              </div>
              <div className='w-1/2'>
                <InputText label='Protocol' name='protocol' value={field.value} disabled />
              </div>
            </div>
          )}
        />

        <FormField
          control={form.control}
          name='scopeType'
          render={({ field }) => (
            <div className='flex items-start justify-between px-8 py-4 border-t'>
              <div className='w-1/3'>
                <p className='text-sm font-medium'>Type</p>
                <p className='text-sm text-muted-foreground mt-0.5'>
                  Define how this scope is assigned to clients.
                </p>
              </div>
              <div className='w-1/2'>
                <FormItem>
                  <FormControl>
                    <Select onValueChange={field.onChange} value={field.value}>
                      <SelectTrigger>
                        <SelectValue placeholder='Select type' />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value='optional'>Optional</SelectItem>
                        <SelectItem value='default'>Default</SelectItem>
                      </SelectContent>
                    </Select>
                  </FormControl>
                </FormItem>
              </div>
            </div>
          )}
        />
      </div>

      <FloatingActionBar
        show={formIsValid ?? false}
        title='Create Client Scope'
        onCancel={handleBack}
        description='Create a new client scope with the specified details.'
        actions={[{ label: 'Create', onClick: handleSubmit }]}
      />
    </div>
  )
}
