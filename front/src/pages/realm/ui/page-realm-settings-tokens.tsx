import { UseFormReturn } from 'react-hook-form'
import { RealmTokenSettingsSchema } from '../feature/page-realm-settings-tokens-feature'
import { Form, FormControl, FormField, FormItem } from '@/components/ui/form'
import { InputText } from '@/components/ui/input-text'
import FloatingActionBar from '@/components/ui/floating-action-bar'

export interface PageRealmSettingsTokensProps {
  form: UseFormReturn<RealmTokenSettingsSchema>
  hasChanges: boolean
  handleSubmit: (values: RealmTokenSettingsSchema) => void
}

function formatDuration(seconds: number): string {
  if (seconds < 60) return `${seconds}s`
  if (seconds < 3600) return `${Math.floor(seconds / 60)}min`
  if (seconds < 86400) return `${Math.floor(seconds / 3600)}h`
  return `${Math.floor(seconds / 86400)}d`
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
                    Duration in seconds before access tokens expire. Between 60s and 86400s (24h).
                  </p>
                </div>
                <div className='w-1/2'>
                  <FormItem>
                    <FormControl>
                      <InputText
                        label={`Access Token Lifetime (${formatDuration(field.value)})`}
                        value={field.value}
                        name='accessTokenLifetime'
                        type='number'
                        onChange={(val) => field.onChange(val)}
                        error={fieldState.error?.message}
                      />
                    </FormControl>
                  </FormItem>
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
                    Duration in seconds before refresh tokens expire. Between 300s (5min) and 2592000s (30d).
                  </p>
                </div>
                <div className='w-1/2'>
                  <FormItem>
                    <FormControl>
                      <InputText
                        label={`Refresh Token Lifetime (${formatDuration(field.value)})`}
                        value={field.value}
                        name='refreshTokenLifetime'
                        type='number'
                        onChange={(val) => field.onChange(val)}
                        error={fieldState.error?.message}
                      />
                    </FormControl>
                  </FormItem>
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
                    Duration in seconds before ID tokens expire. Between 60s and 86400s (24h).
                  </p>
                </div>
                <div className='w-1/2'>
                  <FormItem>
                    <FormControl>
                      <InputText
                        label={`ID Token Lifetime (${formatDuration(field.value)})`}
                        value={field.value}
                        name='idTokenLifetime'
                        type='number'
                        onChange={(val) => field.onChange(val)}
                        error={fieldState.error?.message}
                      />
                    </FormControl>
                  </FormItem>
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
                    Duration in seconds before temporary tokens (e.g. password reset) expire. Between 60s and 86400s (24h).
                  </p>
                </div>
                <div className='w-1/2'>
                  <FormItem>
                    <FormControl>
                      <InputText
                        label={`Temporary Token Lifetime (${formatDuration(field.value)})`}
                        value={field.value}
                        name='temporaryTokenLifetime'
                        type='number'
                        onChange={(val) => field.onChange(val)}
                        error={fieldState.error?.message}
                      />
                    </FormControl>
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
          description='You have unsaved changes in your token settings.'
          onCancel={form.reset}
        />
      </div>
    </Form>
  )
}
