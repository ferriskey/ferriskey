import { InputText } from '@/components/ui/input-text'
import ManageRedirectUris from '../components/manage-redirect-uris'
import ManagePostLogoutRedirectUris from '../components/manage-post-logout-redirect-uris'
import { FormControl, FormField, FormItem, FormLabel } from '@/components/ui/form'
import { Switch } from '@/components/ui/switch'
import { UseFormReturn } from 'react-hook-form'
import { UpdateClientSchema } from '../schemas/update-client.schema'
import FloatingActionBar from '@/components/ui/floating-action-bar'
import { Schemas } from '@/api/api.client.ts'
import Client = Schemas.Client

export interface PageClientSettingsProps {
  client: Client
  form: UseFormReturn<UpdateClientSchema>
  handleSubmit: () => void
  hasChanges: boolean
  refetch: () => void
}

export default function PageClientSettings({
  client,
  form,
  handleSubmit,
  hasChanges,
  refetch,
}: PageClientSettingsProps) {
  return (
    <div className='flex flex-col gap-8'>
      {/* General Settings section */}
      <div className='flex flex-col gap-1'>
        <div className='mb-4'>
          <p className='text-xs text-muted-foreground mb-0.5'>Client configuration</p>
          <h2 className='text-base font-semibold'>General Settings</h2>
        </div>

        {/* Client Name */}
        <FormField
          control={form.control}
          name='name'
          render={({ field }) => (
            <div className='flex items-start justify-between py-4 border-t'>
              <div className='w-1/3'>
                <p className='text-sm font-medium'>Client Name</p>
                <p className='text-sm text-muted-foreground mt-0.5'>Display name shown in the UI.</p>
              </div>
              <div className='w-1/2'>
                <InputText
                  label='Client Name'
                  value={field.value}
                  name='client_name'
                  onChange={field.onChange}
                />
              </div>
            </div>
          )}
        />

        {/* Client ID */}
        <FormField
          control={form.control}
          name='clientId'
          render={({ field }) => (
            <div className='flex items-start justify-between py-4 border-t'>
              <div className='w-1/3'>
                <p className='text-sm font-medium'>Client ID</p>
                <p className='text-sm text-muted-foreground mt-0.5'>Unique identifier for this client.</p>
              </div>
              <div className='w-1/2'>
                <InputText
                  label='Client ID'
                  value={field.value}
                  name='client_id'
                  onChange={field.onChange}
                />
              </div>
            </div>
          )}
        />

        {/* Enabled */}
        <FormField
          name='enabled'
          control={form.control}
          render={({ field }) => (
            <div className='flex items-center justify-between py-4 border-t'>
              <div className='w-1/3'>
                <p className='text-sm font-medium'>Client Enabled</p>
                <p className='text-sm text-muted-foreground mt-0.5'>
                  Disabled clients cannot authenticate users.
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

      {/* Access Settings section */}
      <div className='flex flex-col gap-1'>
        <div className='mb-4'>
          <p className='text-xs text-muted-foreground mb-0.5'>Access configuration</p>
          <h2 className='text-base font-semibold'>Access Settings</h2>
        </div>

        <div className='flex items-start justify-between py-4 border-t'>
          <div className='w-1/3'>
            <p className='text-sm font-medium'>Redirect URIs</p>
            <p className='text-sm text-muted-foreground mt-0.5'>
              Allowed redirect URIs after authentication.
            </p>
          </div>
          <div className='w-1/2'>
            <ManageRedirectUris redirectUris={client.redirect_uris ?? []} refetch={refetch} />
          </div>
        </div>
      </div>

      {/* Logout Settings section */}
      <div className='flex flex-col gap-1'>
        <div className='mb-4'>
          <p className='text-xs text-muted-foreground mb-0.5'>Logout configuration</p>
          <h2 className='text-base font-semibold'>Logout Settings</h2>
        </div>

        <div className='flex items-start justify-between py-4 border-t'>
          <div className='w-1/3'>
            <p className='text-sm font-medium'>Post-Logout Redirect URIs</p>
            <p className='text-sm text-muted-foreground mt-0.5'>
              Allowed redirect URIs after logout.
            </p>
          </div>
          <div className='w-1/2'>
            <ManagePostLogoutRedirectUris />
          </div>
        </div>
      </div>

      <FloatingActionBar
        show={hasChanges}
        title='Save Changes'
        actions={[
          {
            label: 'Save',
            variant: 'default',
            onClick: form.handleSubmit(handleSubmit),
          },
        ]}
        description='Save changes to the client settings. Make sure to review all changes before saving.'
        onCancel={() => form.reset()}
      />
    </div>
  )
}
