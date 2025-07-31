import { Client } from '@/api/core.interface'
import BlockContent from '@/components/ui/block-content'
import { InputText } from '@/components/ui/input-text'
import ManageRedirectUris from '../components/manage-redirect-uris'
import { FormControl, FormDescription, FormField, FormItem, FormLabel } from '@/components/ui/form'
import { Switch } from '@/components/ui/switch'
import { UseFormReturn } from 'react-hook-form'
import { UpdateClientSchema } from '../schemas/update-client.schema'
import FloatingActionBar from '@/components/ui/floating-action-bar'

export interface PageClientSettingsProps {
  client: Client
  form: UseFormReturn<UpdateClientSchema>
  handleSubmit: () => void
  hasChanges: boolean
}

export default function PageClientSettings({
  client,
  form,
  handleSubmit,
  hasChanges,
}: PageClientSettingsProps) {
  return (
    <div>
      <div className="flex flex-col gap-4">
        <BlockContent title="General Settings" className="w-full md:w-2/3 2xl:w-1/3">
          <div className="flex flex-col gap-4">
            <FormField
              control={form.control}
              name="name"
              render={({ field }) => (
                <InputText
                  label="Client Name"
                  value={field.value}
                  name="client_name"
                  onChange={field.onChange}
                />
              )}
            />

            <FormField
              control={form.control}
              name="clientId"
              render={({ field }) => (
                <InputText
                  label="Client ID"
                  value={field.value}
                  name="client_id"
                  onChange={field.onChange}
                />
              )}
            />

            <FormField
              name="enabled"
              control={form.control}
              render={({ field }) => (
                <FormItem className="flex flex-row items-center justify-between gap-5 rounded-lg  border p-3 shadow-sm">
                  <div className="space-y-0.5">
                    <FormLabel>Client Enabled</FormLabel>
                    <FormDescription>
                      Toggle to enable or disable the client. Disabled clients cannot authenticate
                      users.
                    </FormDescription>
                  </div>

                  <FormControl>
                    <Switch checked={field.value} onCheckedChange={field.onChange} />
                  </FormControl>
                </FormItem>
              )}
            />
          </div>
        </BlockContent>

        <BlockContent title="Access Settings" className="w-full md:w-2/3 2xl:w-1/3">
          <div>
            <ManageRedirectUris redirectUris={client.redirect_uris ?? []} />
          </div>
        </BlockContent>
      </div>

      <FloatingActionBar
        show={hasChanges}
        title="Save Changes"
        actions={[
          {
            label: 'Save',
            variant: 'default',
            onClick: form.handleSubmit(handleSubmit),
          },
        ]}
        description="Save changes to the client settings. Make sure to review all changes before saving."
        onCancel={() => form.reset()}
      />
    </div>
  )
}
