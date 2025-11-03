import { UseFormReturn } from 'react-hook-form'
import { RealmLoginSettingsSchema } from '../feature/page-realm-settings-login-feature'
import { Form, FormField } from '@/components/ui/form'
import BlockContent from '@/components/ui/block-content'
import { FormSwitch } from '@/components/ui/switch'
import FloatingActionBar from '@/components/ui/floating-action-bar'

export interface PageRealmSettingsLoginProps {
  form: UseFormReturn<RealmLoginSettingsSchema>
  hasChanges: boolean
  handleSubmit: (values: RealmLoginSettingsSchema) => void
}

export default function PageRealmSettingsLogin({ form, hasChanges, handleSubmit }: PageRealmSettingsLoginProps) {

  return (
    <div className='flex flex-col gap-6'>
      <Form {...form}>
        <BlockContent title='Login Settings' className='w-full md:w-2/3 2xl:w-1/2'>
          <div className='flex flex-col gap-4'>
            <FormField
              control={form.control}
              name='userRegistration'
              render={({ field }) => (
                <FormSwitch
                  label='User Registration'
                  description='Allow users to register themselves through the login page'
                  checked={field.value}
                  onChange={field.onChange}
                />
              )}
            />

            <FormField
              control={form.control}
              name='forgotPassword'
              render={({ field }) => (
                <FormSwitch
                  label='Forgot Password'
                  description='Show forgot password link on login page'
                  checked={field.value}
                  onChange={field.onChange}
                />
              )}
            />

            <FormField
              control={form.control}
              name='rememberMe'
              render={({ field }) => (
                <FormSwitch
                  label='Remember Me'
                  description='Show remember me checkbox on login page'
                  checked={field.value}
                  onChange={field.onChange}
                />
              )}
            />

          </div>
        </BlockContent>
      </Form>

      <FloatingActionBar
        show={hasChanges}
        title='Save Changes'
        actions={[
          {
            label: 'Save',
            variant: 'default',
            onClick: () => form.handleSubmit(handleSubmit)(),
          }
        ]}
        description='You have unsaved changes in your login settings.'
        onCancel={() => form.reset()}

      />

    </div>
  )

}
