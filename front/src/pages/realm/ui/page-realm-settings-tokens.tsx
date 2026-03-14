import { UseFormReturn } from 'react-hook-form'
import { RealmTokenSettingsSchema } from '../feature/page-realm-settings-tokens-feature'
import { Form, FormField } from '@/components/ui/form'
import { DurationInput } from '@/components/ui/duration-input'
import FloatingActionBar from '@/components/ui/floating-action-bar'

export interface PageRealmSettingsTokensProps {
  form: UseFormReturn<RealmTokenSettingsSchema>
  hasChanges: boolean
  handleSubmit: (values: RealmTokenSettingsSchema) => void
}

export default function PageRealmSettingsTokens({ form, hasChanges, handleSubmit }: PageRealmSettingsTokensProps) {
  return (
    <Form {...form}>
      <div className='flex flex-col gap-8'>
        <div className='flex flex-col gap-1'>
          <div className='mb-4'>
            <p className='text-xs text-muted-foreground mb-0.5'>Default token lifetimes for this realm</p>
            <h2 className='text-base font-semibold'>Token Lifetimes</h2>
          </div>

          <FormField
            control={form.control}
            name='accessTokenLifetime'
            render={({ field, fieldState }) => (
              <div className='flex items-start justify-between py-4 border-t'>
                <div className='w-1/3'>
                  <p className='text-sm font-medium'>Access Token Lifetime</p>
                  <p className='text-sm text-muted-foreground mt-0.5'>
                    How long access tokens remain valid.
                  </p>
                </div>
                <div className='w-1/2'>
                  <DurationInput
                    label='Access Token Lifetime'
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
            name='refreshTokenLifetime'
            render={({ field, fieldState }) => (
              <div className='flex items-start justify-between py-4 border-t'>
                <div className='w-1/3'>
                  <p className='text-sm font-medium'>Refresh Token Lifetime</p>
                  <p className='text-sm text-muted-foreground mt-0.5'>
                    How long refresh tokens remain valid.
                  </p>
                </div>
                <div className='w-1/2'>
                  <DurationInput
                    label='Refresh Token Lifetime'
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
            name='idTokenLifetime'
            render={({ field, fieldState }) => (
              <div className='flex items-start justify-between py-4 border-t'>
                <div className='w-1/3'>
                  <p className='text-sm font-medium'>ID Token Lifetime</p>
                  <p className='text-sm text-muted-foreground mt-0.5'>
                    How long ID tokens remain valid.
                  </p>
                </div>
                <div className='w-1/2'>
                  <DurationInput
                    label='ID Token Lifetime'
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
            name='temporaryTokenLifetime'
            render={({ field, fieldState }) => (
              <div className='flex items-start justify-between py-4 border-t'>
                <div className='w-1/3'>
                  <p className='text-sm font-medium'>Temporary Token Lifetime</p>
                  <p className='text-sm text-muted-foreground mt-0.5'>
                    How long temporary tokens (e.g. password reset) remain valid.
                  </p>
                </div>
                <div className='w-1/2'>
                  <DurationInput
                    label='Temporary Token Lifetime'
                    value={field.value}
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
          actions={[{ label: 'Save', variant: 'default', onClick: () => form.handleSubmit(handleSubmit)() }]}
          description='You have unsaved changes in your token settings.'
          onCancel={form.reset}
        />
      </div>
    </Form>
  )
}
