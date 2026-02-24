import { UseFormReturn } from 'react-hook-form'
import { RealmLoginSettingsSchema } from '../feature/page-realm-settings-login-feature'
import { Form, FormControl, FormField, FormItem, FormLabel } from '@/components/ui/form'
import { Switch } from '@/components/ui/switch'
import FloatingActionBar from '@/components/ui/floating-action-bar'

export interface PageRealmSettingsLoginProps {
  form: UseFormReturn<RealmLoginSettingsSchema>
  hasChanges: boolean
  handleSubmit: (values: RealmLoginSettingsSchema) => void
}

export default function PageRealmSettingsLogin({ form, hasChanges, handleSubmit }: PageRealmSettingsLoginProps) {
  return (
    <Form {...form}>
      <div className='flex flex-col gap-8'>
        <div className='flex flex-col gap-1'>
          <div className='mb-4'>
            <p className='text-xs text-muted-foreground mb-0.5'>Login page configuration</p>
            <h2 className='text-base font-semibold'>Login Settings</h2>
          </div>

          <FormField
            control={form.control}
            name='userRegistration'
            render={({ field }) => (
              <div className='flex items-center justify-between py-4 border-t'>
                <div className='w-1/3'>
                  <p className='text-sm font-medium'>User Registration</p>
                  <p className='text-sm text-muted-foreground mt-0.5'>Allow users to register themselves through the login page.</p>
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
            name='forgotPassword'
            render={({ field }) => (
              <div className='flex items-center justify-between py-4 border-t'>
                <div className='w-1/3'>
                  <p className='text-sm font-medium'>Forgot Password</p>
                  <p className='text-sm text-muted-foreground mt-0.5'>Show a forgot password link on the login page.</p>
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
            name='rememberMe'
            render={({ field }) => (
              <div className='flex items-center justify-between py-4 border-t'>
                <div className='w-1/3'>
                  <p className='text-sm font-medium'>Remember Me</p>
                  <p className='text-sm text-muted-foreground mt-0.5'>Show a remember me checkbox on the login page.</p>
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

        <FloatingActionBar
          show={hasChanges}
          title='Save Changes'
          actions={[{ label: 'Save', variant: 'default', onClick: () => form.handleSubmit(handleSubmit)() }]}
          description='You have unsaved changes in your login settings.'
          onCancel={form.reset}
        />
      </div>
    </Form>
  )
}
