import { Button } from '@/components/ui/button'
import { ArrowLeft } from 'lucide-react'
import { UseFormReturn } from 'react-hook-form'
import { Heading } from '@/components/ui/heading'
import BlockContent from '@/components/ui/block-content'
import { FormControl, FormDescription, FormField, FormItem, FormLabel } from '@/components/ui/form'
import { InputText } from '@/components/ui/input-text'
import { Switch } from '@/components/ui/switch'
import FloatingActionBar from '@/components/ui/floating-action-bar'
import { Badge } from '@/components/ui/badge'
import { Skeleton } from '@/components/ui/skeleton'
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

function ProviderTypeBadge({ type }: { type: string }) {
  const label = providerTypeLabels[type.toLowerCase()] ?? type
  return <Badge variant='outline'>{label}</Badge>
}

function LoadingSkeleton() {
  return (
    <div className='flex flex-col p-4 gap-4'>
      <div className='flex items-center gap-3'>
        <Skeleton className='h-8 w-8' />
        <Skeleton className='h-4 w-32' />
      </div>
      <Skeleton className='h-8 w-64' />
      <Skeleton className='h-64 w-full lg:w-1/2' />
    </div>
  )
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
    return <LoadingSkeleton />
  }

  if (!provider) {
    return (
      <div className='flex flex-col p-4 gap-4'>
        <div className='flex items-center gap-3'>
          <Button variant='ghost' size='icon' onClick={handleBack}>
            <ArrowLeft className='h-3 w-3' />
          </Button>
          <span className='text-gray-500 text-sm font-medium'>Back to providers</span>
        </div>
        <div className='text-center py-12'>
          <p className='text-muted-foreground'>Provider not found</p>
        </div>
      </div>
    )
  }

  return (
    <div className='flex flex-col p-4 gap-4'>
      <div className='flex items-center gap-3'>
        <Button variant='ghost' size='icon' onClick={handleBack}>
          <ArrowLeft className='h-3 w-3' />
        </Button>
        <span className='text-gray-500 text-sm font-medium'>Back to providers</span>
      </div>

      <div className='flex flex-col mb-4'>
        <div className='flex items-center gap-3'>
          <Heading size={3} className='text-gray-800'>
            {provider.display_name ?? provider.alias}
          </Heading>
          <ProviderTypeBadge type={provider.provider_id} />
          <Badge variant={provider.enabled ? 'default' : 'secondary'}>
            {provider.enabled ? 'Enabled' : 'Disabled'}
          </Badge>
        </div>
        <p className='text-sm text-gray-500 mt-1'>
          Alias: {provider.alias}
        </p>
      </div>

      <div className='flex flex-col gap-4'>
        <BlockContent title='General Settings' className='w-full md:w-2/3 2xl:w-1/3'>
          <div className='flex flex-col gap-4'>
            <InputText
              label='Alias'
              value={provider.alias}
              name='alias'
              disabled
            />

            <FormField
              control={form.control}
              name='displayName'
              render={({ field }) => (
                <InputText
                  label='Display Name'
                  value={field.value}
                  name='displayName'
                  onChange={field.onChange}
                />
              )}
            />

            <FormField
              name='enabled'
              control={form.control}
              render={({ field }) => (
                <FormItem className='flex flex-row items-center justify-between gap-5 rounded-lg border p-3 shadow-sm'>
                  <div className='space-y-0.5'>
                    <FormLabel>Enabled</FormLabel>
                    <FormDescription>
                      Allow users to authenticate using this provider.
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

        <BlockContent title='Configuration' className='w-full md:w-2/3 2xl:w-1/3'>
          <div className='flex flex-col gap-4'>
            {(() => {
              const config =
                provider.config && typeof provider.config === 'object'
                  ? (provider.config as Record<string, unknown>)
                  : {}
              return Object.entries(config).map(([key, value]) => (
                <InputText
                  key={key}
                  label={key.split('_').map(w => w.charAt(0).toUpperCase() + w.slice(1)).join(' ')}
                  value={
                    key.includes('secret') || key.includes('credential')
                      ? '********'
                      : String(value ?? '')
                  }
                  name={key}
                  disabled
                />
              ))
            })()}
            {(!provider.config ||
              typeof provider.config !== 'object' ||
              Object.keys(provider.config as Record<string, unknown>).length === 0) && (
              <p className='text-sm text-muted-foreground'>No configuration settings</p>
            )}
          </div>
        </BlockContent>

        <BlockContent title='Metadata' className='w-full md:w-2/3 2xl:w-1/3'>
          <div className='grid grid-cols-2 gap-4 text-sm'>
            <div>
              <p className='text-muted-foreground'>Internal ID</p>
              <p className='font-mono text-xs'>{provider.internal_id}</p>
            </div>
            <div>
              <p className='text-muted-foreground'>Provider ID</p>
              <p className='font-mono text-xs'>{provider.provider_id}</p>
            </div>
            <div>
              <p className='text-muted-foreground'>First Broker Flow</p>
              <p>{provider.first_broker_login_flow_alias ?? '—'}</p>
            </div>
            <div>
              <p className='text-muted-foreground'>Post Broker Flow</p>
              <p>{provider.post_broker_login_flow_alias ?? '—'}</p>
            </div>
          </div>
        </BlockContent>

        <BlockContent title='Danger Zone' className='w-full md:w-2/3 2xl:w-1/3'>
          <div className='flex items-center justify-between'>
            <div>
              <p className='font-medium'>Delete Provider</p>
              <p className='text-sm text-muted-foreground'>
                Permanently delete this identity provider. This action cannot be undone.
              </p>
            </div>
            <Button variant='destructive' onClick={handleDelete}>
              Delete
            </Button>
          </div>
        </BlockContent>
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
        description='Save changes to the provider settings.'
        onCancel={() => form.reset()}
      />
    </div>
  )
}
