import { DangerZone } from '@/components/danger-zone'
import FloatingActionBar from '@/components/ui/floating-action-bar'
import { FormControl, FormField, FormItem, FormLabel } from '@/components/ui/form'
import { InputText } from '@/components/ui/input-text'
import { Switch } from '@/components/ui/switch'
import { Schemas } from '@/api/api.client.ts'
import { UseFormReturn } from 'react-hook-form'
import { UpdateOrganizationSchema } from '../schemas/update-organization.schema'
import Organization = Schemas.Organization

export interface PageOrganizationSettingsProps {
  organization: Organization
  form: UseFormReturn<UpdateOrganizationSchema>
  handleSubmit: () => void
  hasChanges: boolean
  onDelete: () => void
}

export default function PageOrganizationSettings({
  form,
  handleSubmit,
  hasChanges,
  organization,
  onDelete,
}: PageOrganizationSettingsProps) {
  return (
    <div className='flex flex-col gap-8'>
      {/* General Settings */}
      <div className='flex flex-col gap-1'>
        <div className='mb-4'>
          <p className='text-xs text-muted-foreground mb-0.5'>Organization configuration</p>
          <h2 className='text-base font-semibold'>General Settings</h2>
        </div>

        <FormField
          control={form.control}
          name='name'
          render={({ field, fieldState }) => (
            <div className='flex items-start justify-between py-4 border-t'>
              <div className='w-1/3'>
                <p className='text-sm font-medium'>Name</p>
                <p className='text-sm text-muted-foreground mt-0.5'>
                  Human-readable display name for this organization.
                </p>
              </div>
              <div className='w-1/2'>
                <InputText
                  label='Name'
                  name='name'
                  value={field.value}
                  onChange={field.onChange}
                  error={fieldState.error?.message}
                />
              </div>
            </div>
          )}
        />

        <FormField
          control={form.control}
          name='alias'
          render={({ field, fieldState }) => (
            <div className='flex items-start justify-between py-4 border-t'>
              <div className='w-1/3'>
                <p className='text-sm font-medium'>Alias</p>
                <p className='text-sm text-muted-foreground mt-0.5'>
                  Stable URL-safe identifier used for lookups and routing.
                </p>
              </div>
              <div className='w-1/2'>
                <InputText
                  label='Alias'
                  name='alias'
                  value={field.value}
                  onChange={field.onChange}
                  error={fieldState.error?.message}
                />
              </div>
            </div>
          )}
        />

        <FormField
          name='enabled'
          control={form.control}
          render={({ field }) => (
            <div className='flex items-center justify-between py-4 border-t'>
              <div className='w-1/3'>
                <p className='text-sm font-medium'>Enabled</p>
                <p className='text-sm text-muted-foreground mt-0.5'>
                  Disabled organizations are hidden from end-user flows.
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
      </div>

      {/* Contact & Routing */}
      <div className='flex flex-col gap-1'>
        <div className='mb-4'>
          <p className='text-xs text-muted-foreground mb-0.5'>Optional metadata</p>
          <h2 className='text-base font-semibold'>Contact & Routing</h2>
        </div>

        <FormField
          control={form.control}
          name='domain'
          render={({ field, fieldState }) => (
            <div className='flex items-start justify-between py-4 border-t'>
              <div className='w-1/3'>
                <p className='text-sm font-medium'>Domain</p>
                <p className='text-sm text-muted-foreground mt-0.5'>
                  Primary domain associated with this organization (e.g. acme.com).
                </p>
              </div>
              <div className='w-1/2'>
                <InputText
                  label='Domain'
                  name='domain'
                  value={field.value ?? ''}
                  onChange={field.onChange}
                  error={fieldState.error?.message}
                />
              </div>
            </div>
          )}
        />

        <FormField
          control={form.control}
          name='redirectUrl'
          render={({ field, fieldState }) => (
            <div className='flex items-start justify-between py-4 border-t'>
              <div className='w-1/3'>
                <p className='text-sm font-medium'>Redirect URL</p>
                <p className='text-sm text-muted-foreground mt-0.5'>
                  Default redirect destination for this organization's authentication flow.
                </p>
              </div>
              <div className='w-1/2'>
                <InputText
                  label='Redirect URL'
                  name='redirectUrl'
                  value={field.value ?? ''}
                  onChange={field.onChange}
                  error={fieldState.error?.message}
                />
              </div>
            </div>
          )}
        />

        <FormField
          control={form.control}
          name='description'
          render={({ field, fieldState }) => (
            <div className='flex items-start justify-between py-4 border-t'>
              <div className='w-1/3'>
                <p className='text-sm font-medium'>Description</p>
                <p className='text-sm text-muted-foreground mt-0.5'>
                  Optional notes visible to admins.
                </p>
              </div>
              <div className='w-1/2'>
                <InputText
                  label='Description'
                  name='description'
                  value={field.value ?? ''}
                  onChange={field.onChange}
                  error={fieldState.error?.message}
                />
              </div>
            </div>
          )}
        />
      </div>

      <FloatingActionBar
        show={hasChanges}
        title='Save Changes'
        description='Review your changes before saving.'
        actions={[{ label: 'Save', variant: 'default', onClick: handleSubmit }]}
        onCancel={() => form.reset()}
      />

      <DangerZone
        label='Delete this organization'
        description='All members, attributes, and configuration will be permanently removed.'
        buttonLabel='Delete organization'
        confirmTitle='Delete organization'
        confirmDescription={`This will permanently delete "${organization.name}" and all its associated data.`}
        onConfirm={onDelete}
      />
    </div>
  )
}
