import { Schemas } from '@/api/api.client'
import { Button } from '@/components/ui/button'
import { FormControl, FormField, FormItem } from '@/components/ui/form'
import { InputText } from '@/components/ui/input-text'
import FloatingActionBar from '@/components/ui/floating-action-bar'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { useState } from 'react'
import { UseFormReturn } from 'react-hook-form'
import { UpdateClientScopeSchema } from '../schemas/update-client-scope.schema'
import { ConfirmDeleteAlert } from '@/components/confirm-delete-alert'

import ClientScope = Schemas.ClientScope

interface PageClientScopeDetailProps {
  scope: ClientScope
  isLoading?: boolean
  form: UseFormReturn<UpdateClientScopeSchema>
  formIsValid: boolean
  isPending: boolean
  handleSubmit: () => void
  handleReset: () => void
  handleDelete: () => void
  isDeleting?: boolean
}

export default function PageClientScopeDetail({
  scope,
  isLoading,
  form,
  formIsValid,
  isPending,
  handleSubmit,
  handleReset,
  handleDelete,
}: PageClientScopeDetailProps) {
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false)
  if (isLoading) {
    return <div className='text-sm text-muted-foreground'>Loading...</div>
  }

  return (
    <div className='flex flex-col gap-8'>
      <div className='flex flex-col gap-1'>
        <div className='mb-4'>
          <p className='text-xs text-muted-foreground mb-0.5'>Client scope details</p>
          <h2 className='text-base font-semibold'>General Information</h2>
        </div>

        <FormField
          control={form.control}
          name='name'
          render={({ field }) => (
            <div className='flex items-start justify-between py-4 border-t'>
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
            <div className='flex items-start justify-between py-4 border-t'>
              <div className='w-1/3'>
                <p className='text-sm font-medium'>Description</p>
                <p className='text-sm text-muted-foreground mt-0.5'>Optional description for this client scope.</p>
              </div>
              <div className='w-1/2'>
                <InputText label='Description' {...field} />
              </div>
            </div>
          )}
        />

        <div className='flex items-start justify-between py-4 border-t'>
          <div className='w-1/3'>
            <p className='text-sm font-medium'>Protocol</p>
            <p className='text-sm text-muted-foreground mt-0.5'>Only OpenID Connect is currently supported.</p>
          </div>
          <div className='w-1/2'>
            <InputText label='Protocol' name='protocol' value={scope.protocol} disabled />
          </div>
        </div>

        <FormField
          control={form.control}
          name='scopeType'
          render={({ field }) => (
            <div className='flex items-start justify-between py-4 border-t'>
              <div className='w-1/3'>
                <p className='text-sm font-medium'>Type</p>
                <p className='text-sm text-muted-foreground mt-0.5'>Define how this scope is assigned to clients.</p>
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

        <div className='flex items-start justify-between py-4 border-t'>
          <div className='w-1/3'>
            <p className='text-sm font-medium'>Created At</p>
          </div>
          <div className='w-1/2'>
            <p className='text-sm text-foreground'>{new Date(scope.created_at).toLocaleString()}</p>
          </div>
        </div>

        <div className='flex items-start justify-between py-4 border-t'>
          <div className='w-1/3'>
            <p className='text-sm font-medium'>Updated At</p>
          </div>
          <div className='w-1/2'>
            <p className='text-sm text-foreground'>{new Date(scope.updated_at).toLocaleString()}</p>
          </div>
        </div>
      </div>

      <div className='flex flex-col gap-1'>
        <div className='mb-4'>
          <p className='text-xs text-muted-foreground mb-0.5'>Client scope attributes</p>
          <h2 className='text-base font-semibold'>Attributes</h2>
        </div>
        {(scope.attributes?.length ?? 0) === 0 ? (
          <div className='py-4 border-t text-sm text-muted-foreground'>No attributes configured.</div>
        ) : (
          scope.attributes?.map((attribute) => (
            <div key={attribute.id} className='py-4 border-t flex items-start justify-between gap-4'>
              <div className='w-1/3'>
                <p className='text-sm font-medium'>{attribute.name}</p>
              </div>
              <div className='w-1/2'>
                <p className='text-sm text-foreground break-words'>{attribute.value || '-'}</p>
              </div>
            </div>
          ))
        )}
      </div>

      <div className='flex flex-col gap-1'>
        <div className='mb-4'>
          <p className='text-xs text-muted-foreground mb-0.5'>Protocol mapper configuration</p>
          <h2 className='text-base font-semibold'>Protocol Mappers</h2>
        </div>
        {(scope.protocol_mappers?.length ?? 0) === 0 ? (
          <div className='py-4 border-t text-sm text-muted-foreground'>No protocol mappers configured.</div>
        ) : (
          scope.protocol_mappers?.map((mapper) => (
            <div key={mapper.id} className='py-4 border-t flex items-start justify-between gap-4'>
              <div className='w-1/3'>
                <p className='text-sm font-medium'>{mapper.name}</p>
                <p className='text-xs text-muted-foreground mt-0.5'>mapper_type: {mapper.mapper_type}</p>
              </div>
              <div className='w-1/2 text-sm text-muted-foreground break-words'>
                id: {mapper.id}
              </div>
            </div>
          ))
        )}
      </div>

      {/* Danger Zone */}
      <div className='flex flex-col gap-1'>
        <div className='mb-4'>
          <p className='text-xs text-destructive/70 mb-0.5'>Irreversible actions</p>
          <h2 className='text-base font-semibold text-destructive'>Danger Zone</h2>
        </div>

        <div className='flex items-center justify-between py-4 border-t border-destructive/20'>
          <div className='w-2/3'>
            <p className='text-sm font-medium'>Delete this client scope</p>
            <p className='text-sm text-muted-foreground mt-0.5'>
              Once deleted, all associated protocol mappers and client mappings will be permanently removed.
            </p>
          </div>
          <Button variant='destructive' onClick={() => setDeleteDialogOpen(true)}>
            Delete scope
          </Button>
        </div>
      </div>

      <ConfirmDeleteAlert
        open={deleteDialogOpen}
        title='Delete client scope'
        description={`This will permanently delete the scope "${scope.name}" and all its associated protocol mappers and client mappings.`}
        onConfirm={() => {
          handleDelete()
          setDeleteDialogOpen(false)
        }}
        onCancel={() => setDeleteDialogOpen(false)}
      />

      <FloatingActionBar
        show={formIsValid}
        title='Unsaved Changes'
        description='You have unsaved changes to this client scope.'
        onCancel={handleReset}
        cancelLabel='Discard'
        actions={[{ label: isPending ? 'Saving...' : 'Save Changes', onClick: handleSubmit }]}
      />
    </div>
  )
}
