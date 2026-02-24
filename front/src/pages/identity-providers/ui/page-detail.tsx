import { ArrowLeft } from 'lucide-react'
import { UseFormReturn } from 'react-hook-form'
import { FormControl, FormField, FormItem, FormLabel } from '@/components/ui/form'
import { InputText } from '@/components/ui/input-text'
import { Switch } from '@/components/ui/switch'
import FloatingActionBar from '@/components/ui/floating-action-bar'
import { Skeleton } from '@/components/ui/skeleton'
import { Button } from '@/components/ui/button'
import type { Schemas } from '@/api/api.client'

interface UpdateProviderSchema {
  displayName: string
  enabled: boolean
}

export interface PageDetailProps {
  provider: Schemas.IdentityProviderResponse | null
  isLoading: boolean
  form: UseFormReturn<UpdateProviderSchema>
  handleBack: () => void
  handleSubmit: () => void
  handleDelete: () => void
  hasChanges: boolean
}

const providerTypeLabels: Record<string, string> = {
  oidc: 'OIDC',
  oauth2: 'OAuth2',
  saml: 'SAML',
  ldap: 'LDAP',
}

export default function PageDetail({
  provider,
  isLoading,
  form,
  handleBack,
  handleSubmit,
  handleDelete,
  hasChanges,
}: PageDetailProps) {
  if (isLoading) {
    return (
      <div className='flex flex-col gap-6 p-8'>
        <div className='-mx-8 -mt-8 px-8 pt-8 pb-4 border-b'>
          <Skeleton className='h-3.5 w-24 mb-2' />
          <Skeleton className='h-8 w-64 mb-2' />
          <Skeleton className='h-4 w-48' />
        </div>
        <Skeleton className='h-48 w-full' />
      </div>
    )
  }

  if (!provider) {
    return (
      <div className='flex flex-col gap-6 p-8'>
        <div className='-mx-8 -mt-8 px-8 pt-8 pb-4 border-b'>
          <button
            onClick={handleBack}
            className='flex items-center gap-1.5 text-sm text-muted-foreground hover:text-foreground transition-colors mb-2'
          >
            <ArrowLeft className='h-3.5 w-3.5' />
            Providers
          </button>
        </div>
        <div className='flex items-center justify-center h-24 text-sm text-muted-foreground'>
          Provider not found.
        </div>
      </div>
    )
  }

  const providerTypeLabel = providerTypeLabels[provider.provider_id?.toLowerCase()] ?? provider.provider_id
  const config = provider.config && typeof provider.config === 'object'
    ? (provider.config as Record<string, unknown>)
    : {}

  return (
    <div className='flex flex-col gap-6 p-8'>
      {/* Header */}
      <div className='-mx-8 -mt-8 px-8 pt-8 pb-4 border-b flex items-start justify-between gap-4'>
        <div>
          <button
            onClick={handleBack}
            className='flex items-center gap-1.5 text-sm text-muted-foreground hover:text-foreground transition-colors mb-2'
          >
            <ArrowLeft className='h-3.5 w-3.5' />
            Providers
          </button>
          <h1 className='text-2xl font-bold tracking-tight'>
            {provider.display_name ?? provider.alias}
          </h1>
          <p className='text-sm text-muted-foreground mt-1'>
            Alias: {provider.alias}
          </p>
        </div>
        <div className='flex items-center gap-2 shrink-0'>
          <span className='inline-flex items-center px-2.5 py-0.5 rounded-md border border-primary/40 text-primary text-xs font-mono bg-primary/10'>
            {providerTypeLabel}
          </span>
          <span
            className={`inline-flex items-center px-2.5 py-0.5 rounded-md border text-xs font-mono ${
              provider.enabled
                ? 'border-green-300 text-green-600 bg-green-50 dark:bg-green-500/10 dark:border-green-400/40'
                : 'border-border text-muted-foreground bg-muted/50'
            }`}
          >
            {provider.enabled ? 'enabled' : 'disabled'}
          </span>
        </div>
      </div>

      {/* General Settings */}
      <div className='flex flex-col gap-1'>
        <div className='mb-4'>
          <p className='text-xs text-muted-foreground mb-0.5'>Provider configuration</p>
          <h2 className='text-base font-semibold'>General Settings</h2>
        </div>

        <div className='flex items-start justify-between py-4 border-t'>
          <div className='w-1/3'>
            <p className='text-sm font-medium'>Alias</p>
            <p className='text-sm text-muted-foreground mt-0.5'>Unique identifier for this provider.</p>
          </div>
          <div className='w-1/2'>
            <InputText label='Alias' value={provider.alias} name='alias' disabled />
          </div>
        </div>

        <FormField
          control={form.control}
          name='displayName'
          render={({ field }) => (
            <div className='flex items-start justify-between py-4 border-t'>
              <div className='w-1/3'>
                <p className='text-sm font-medium'>Display Name</p>
                <p className='text-sm text-muted-foreground mt-0.5'>Name shown on the login page.</p>
              </div>
              <div className='w-1/2'>
                <InputText label='Display Name' value={field.value} name='displayName' onChange={field.onChange} />
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
                <p className='text-sm text-muted-foreground mt-0.5'>Allow users to authenticate using this provider.</p>
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

      {/* Configuration */}
      <div className='flex flex-col gap-1'>
        <div className='mb-4'>
          <p className='text-xs text-muted-foreground mb-0.5'>OAuth / OIDC settings</p>
          <h2 className='text-base font-semibold'>Configuration</h2>
        </div>

        {Object.keys(config).length === 0 ? (
          <p className='text-sm text-muted-foreground py-4 border-t'>No configuration settings.</p>
        ) : (
          Object.entries(config).map(([key, value]) => (
            <div key={key} className='flex items-start justify-between py-4 border-t'>
              <div className='w-1/3'>
                <p className='text-sm font-medium'>
                  {key.split('_').map(w => w.charAt(0).toUpperCase() + w.slice(1)).join(' ')}
                </p>
              </div>
              <div className='w-1/2'>
                <InputText
                  label={key.split('_').map(w => w.charAt(0).toUpperCase() + w.slice(1)).join(' ')}
                  value={key.includes('secret') || key.includes('credential') ? '••••••••' : String(value ?? '')}
                  name={key}
                  disabled
                />
              </div>
            </div>
          ))
        )}
      </div>

      {/* Metadata */}
      <div className='flex flex-col gap-1'>
        <div className='mb-4'>
          <p className='text-xs text-muted-foreground mb-0.5'>System information</p>
          <h2 className='text-base font-semibold'>Metadata</h2>
        </div>

        <div className='flex items-start justify-between py-4 border-t'>
          <div className='w-1/3'>
            <p className='text-sm font-medium'>Internal ID</p>
          </div>
          <div className='w-1/2'>
            <p className='text-sm font-mono text-muted-foreground'>{provider.internal_id}</p>
          </div>
        </div>

        <div className='flex items-start justify-between py-4 border-t'>
          <div className='w-1/3'>
            <p className='text-sm font-medium'>Provider ID</p>
          </div>
          <div className='w-1/2'>
            <p className='text-sm font-mono text-muted-foreground'>{provider.provider_id}</p>
          </div>
        </div>

        {provider.first_broker_login_flow_alias && (
          <div className='flex items-start justify-between py-4 border-t'>
            <div className='w-1/3'>
              <p className='text-sm font-medium'>First Broker Flow</p>
            </div>
            <div className='w-1/2'>
              <p className='text-sm text-muted-foreground'>{provider.first_broker_login_flow_alias}</p>
            </div>
          </div>
        )}
      </div>

      {/* Danger zone */}
      <div className='flex flex-col gap-1'>
        <div className='mb-4'>
          <p className='text-xs text-destructive/70 mb-0.5'>Irreversible actions</p>
          <h2 className='text-base font-semibold text-destructive'>Danger Zone</h2>
        </div>

        <div className='flex items-center justify-between py-4 border-t border-destructive/20'>
          <div className='w-2/3'>
            <p className='text-sm font-medium'>Delete this identity provider</p>
            <p className='text-sm text-muted-foreground mt-0.5'>
              All associated data will be permanently removed. This action is irreversible.
            </p>
          </div>
          <Button variant='destructive' onClick={handleDelete}>
            Delete provider
          </Button>
        </div>
      </div>

      <FloatingActionBar
        show={hasChanges}
        title='Save Changes'
        actions={[{ label: 'Save', variant: 'default', onClick: form.handleSubmit(handleSubmit) }]}
        description='Save changes to the provider settings.'
        onCancel={() => form.reset()}
      />
    </div>
  )
}
