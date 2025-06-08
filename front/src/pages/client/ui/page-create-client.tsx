import { Button } from '@/components/ui/button.tsx'
import { ArrowLeft } from 'lucide-react'
import { UseFormReturn } from 'react-hook-form'
import { CreateClientSchema } from '@/pages/client/schemas/create-client.schema.ts'
import { Heading } from '@/components/ui/heading.tsx'
import BlockContent from '@/components/ui/block-content.tsx'
import { FormControl, FormDescription, FormField, FormItem, FormLabel } from '@/components/ui/form.tsx'
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
    <div className="flex flex-col p-4 gap-4">
      <div className="flex items-center gap-3">
        <Button
          variant="ghost"
          size="icon"
          onClick={handleBack}
        >
          <ArrowLeft className="h-3 w-3" />
        </Button>
        <span className="text-gray-500 text-sm font-medium">Back to clients</span>
      </div>

      <div className="flex flex-col mb-4">
        <Heading size={3} className="text-gray-800">
          Create Client
        </Heading>

        <p className="text-sm text-gray-500 mt-1">
          Clients are applications and services that can request authentication of a user.
        </p>
      </div>

      <div className="lg:w-1/3">
        <BlockContent title="Client Details">
          <div className="flex flex-col gap-5">
            <FormField
              control={form.control}
              name="clientId"
              render={({ field }) => (
                <InputText label={"Client ID"} {...field} />
              )}
            />

            <FormField
              control={form.control}
              name="name"
              render={({ field }) => (
                <InputText label={"Name"} {...field} />
              )}
            />

            <FormField
              control={form.control}
              name="enabled"
              render={({ field }) => (
                <FormItem className="flex flex-row items-center justify-between gap-5 rounded-lg border p-3 shadow-sm">
                  <div className="space-y-0.5">
                    <FormLabel>Client Enabled</FormLabel>
                    <FormDescription>
                      Toggle to enable or disable the client. Disabled clients cannot authenticate users.
                    </FormDescription>
                  </div>
                  <FormControl>
                    <Switch
                      checked={field.value}
                      onCheckedChange={field.onChange}
                    />
                  </FormControl>
                </FormItem>
              )}
            />

            <FormField
              control={form.control}
              name="clientAuthentication"
              render={({ field }) => (
                <FormItem className="flex flex-row items-center justify-between gap-5 rounded-lg border p-3 shadow-sm">
                  <div className="space-y-0.5">
                    <FormLabel>Client Authentication</FormLabel>
                    <FormDescription>
                      Toggle to enable or disable client authentication. If enabled, clients must authenticate
                      using a secret or certificate.
                    </FormDescription>
                  </div>
                  <FormControl>
                    <Switch
                      checked={field.value}
                      onCheckedChange={field.onChange}
                    />
                  </FormControl>
                </FormItem>
              )}
            />
          </div>
        </BlockContent>

      </div>

      <FloatingActionBar
        show={formIsValid ?? false}
        title={"Create Client"}
        onCancel={() => {
          handleBack()
        }}
        description={"Create a new client with the specified details."}
        actions={[
          {
            label: "Create",
            onClick: handleSubmit,
          }
        ]}
      />

    </div>
  )
}
