import { UseFormReturn } from 'react-hook-form'
import { RealmLoginSettingsSchema } from '../feature/page-realm-settings-login-feature'
import { Form, FormControl, FormField, FormItem, FormLabel, FormMessage } from '@/components/ui/form'
import { Switch } from '@/components/ui/switch'
import { Input } from '@/components/ui/input'
import FloatingActionBar from '@/components/ui/floating-action-bar'
import { Checkbox } from '@/components/ui/checkbox'

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
            name='emailVerification'
            render={({ field }) => (
              <div className='flex items-center justify-between py-4 border-t'>
                <div className='w-1/3'>
                  <p className='text-sm font-medium'>Email Verification</p>
                  <p className='text-sm text-muted-foreground mt-0.5'>Require users to verify their email address before they can sign in.</p>
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

          <FormField
            control={form.control}
            name='passkey'
            render={({ field }) => (
              <div className='flex items-center justify-between py-4 border-t'>
                <div className='w-1/3'>
                  <p className='text-sm font-medium'>Passkey Authentication</p>
                  <p className='text-sm text-muted-foreground mt-0.5'>Allow users to sign in with a passkey instead of a password.</p>
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
            name='magicLink'
            render={({ field }) => (
              <div className='flex items-center justify-between py-4 border-t'>
                <div className='w-1/3'>
                  <p className='text-sm font-medium'>Magic Link</p>
                  <p className='text-sm text-muted-foreground mt-0.5'>Allow users to sign in via a magic link sent by email.</p>
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

          {form.watch('magicLink') && (
            <FormField
              control={form.control}
              name='magicLinkTtl'
              render={({ field }) => (
                <div className='flex items-center justify-between py-4 border-t'>
                  <div className='w-1/3'>
                    <p className='text-sm font-medium'>Magic Link TTL (minutes)</p>
                    <p className='text-sm text-muted-foreground mt-0.5'>How long the magic link remains valid.</p>
                  </div>
                  <div className='w-1/2'>
                    <FormItem>
                      <FormControl>
                        <Input
                          type='number'
                          min={1}
                          className='w-24'
                          value={field.value}
                          onChange={(e) => field.onChange(Number(e.target.value))}
                        />
                      </FormControl>
                    </FormItem>
                  </div>
                </div>
              )}
            />
          )}

          <FormField
            control={form.control}
            name='loginAliases'
            render={({ field }) => {
              const ORDER: Array<'username' | 'email'> = ['username', 'email']
              const toggle = (alias: 'username' | 'email', checked: boolean) => {
                const next = checked
                  ? ORDER.filter((a) => a === alias || field.value.includes(a))
                  : field.value.filter((a) => a !== alias)
                if (next.length > 0) field.onChange(next)
              }
              return (
                <div className='flex items-center justify-between py-4 border-t'>
                  <div className='w-1/3'>
                    <p className='text-sm font-medium'>Login identifiers</p>
                    <p className='text-sm text-muted-foreground mt-0.5'>
                      Which identifiers users may sign in with. Order sets precedence.
                    </p>
                  </div>
                  <div className='w-1/2 flex flex-col gap-2'>
                    {ORDER.map((alias) => (
                      <FormItem key={alias} className='flex flex-row items-center gap-2'>
                        <FormControl>
                          <Checkbox
                            id={`login-alias-${alias}`}
                            checked={field.value.includes(alias)}
                            onCheckedChange={(c) => toggle(alias, c === true)}
                          />
                        </FormControl>
                        <FormLabel htmlFor={`login-alias-${alias}`} className='!mt-0 text-sm font-normal capitalize'>
                          {alias}
                        </FormLabel>
                      </FormItem>
                    ))}
                    <FormMessage />
                  </div>
                </div>
              )
            }}
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
