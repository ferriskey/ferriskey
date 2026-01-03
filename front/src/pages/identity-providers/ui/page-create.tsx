import { Button } from '@/components/ui/button'
import { ArrowLeft } from 'lucide-react'
import { UseFormReturn } from 'react-hook-form'
import { Heading } from '@/components/ui/heading'
import BlockContent from '@/components/ui/block-content'
import { FormControl, FormDescription, FormField, FormItem, FormLabel } from '@/components/ui/form'
import { InputText } from '@/components/ui/input-text'
import { Switch } from '@/components/ui/switch'
import FloatingActionBar from '@/components/ui/floating-action-bar'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import type { CreateProviderSchema } from '../schemas/create-provider.schema'

export interface PageCreateProps {
  form: UseFormReturn<CreateProviderSchema>
  handleBack: () => void
  handleSubmit: () => void
  formIsValid?: boolean
}

export default function PageCreate({ form, handleBack, handleSubmit, formIsValid }: PageCreateProps) {
  const providerType = form.watch('providerType')

  return (
    <div className='flex flex-col p-4 gap-4'>
      <div className='flex items-center gap-3'>
        <Button
          variant='ghost'
          size='icon'
          onClick={handleBack}
        >
          <ArrowLeft className='h-3 w-3' />
        </Button>
        <span className='text-gray-500 text-sm font-medium'>Back to providers</span>
      </div>

      <div className='flex flex-col mb-4'>
        <Heading size={3} className='text-gray-800'>
          Create Identity Provider
        </Heading>

        <p className='text-sm text-gray-500 mt-1'>
          Configure an external authentication source to enable SSO and federated identity.
        </p>
      </div>

      <div className='lg:w-1/2'>
        <BlockContent title='Basic Settings'>
          <div className='flex flex-col gap-5'>
            <FormField
              control={form.control}
              name='alias'
              render={({ field }) => (
                <InputText
                  label='Alias'
                  {...field}
                  error={form.formState.errors.alias?.message}
                />
              )}
            />

            <FormField
              control={form.control}
              name='displayName'
              render={({ field }) => (
                <InputText
                  label='Display Name'
                  {...field}
                  error={form.formState.errors.displayName?.message}
                />
              )}
            />

            <FormField
              control={form.control}
              name='providerType'
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Provider Type</FormLabel>
                  <Select
                    onValueChange={field.onChange}
                    defaultValue={field.value}
                  >
                    <FormControl>
                      <SelectTrigger>
                        <SelectValue placeholder='Select provider type' />
                      </SelectTrigger>
                    </FormControl>
                    <SelectContent>
                      <SelectItem value='oidc'>OpenID Connect (OIDC)</SelectItem>
                      <SelectItem value='oauth2'>OAuth 2.0</SelectItem>
                      <SelectItem value='saml'>SAML 2.0</SelectItem>
                      <SelectItem value='ldap'>LDAP</SelectItem>
                    </SelectContent>
                  </Select>
                </FormItem>
              )}
            />

            <FormField
              control={form.control}
              name='enabled'
              render={({ field }) => (
                <FormItem className='flex flex-row items-center justify-between gap-5 rounded-lg border p-3 shadow-sm'>
                  <div className='space-y-0.5'>
                    <FormLabel>Enabled</FormLabel>
                    <FormDescription>
                      Allow users to authenticate using this provider.
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

        {(providerType === 'oidc' || providerType === 'oauth2') && (
          <BlockContent title={providerType === 'oidc' ? 'OpenID Connect Settings' : 'OAuth 2.0 Settings'}>
            <div className='flex flex-col gap-5'>
              <FormField
                control={form.control}
                name='clientId'
                render={({ field }) => (
                  <InputText
                    label='Client ID'
                    {...field}
                  />
                )}
              />

              <FormField
                control={form.control}
                name='clientSecret'
                render={({ field }) => (
                  <InputText
                    label='Client Secret'
                    type='password'
                    {...field}
                  />
                )}
              />

              <FormField
                control={form.control}
                name='authorizationUrl'
                render={({ field }) => (
                  <InputText
                    label='Authorization URL'
                    {...field}
                  />
                )}
              />

              <FormField
                control={form.control}
                name='tokenUrl'
                render={({ field }) => (
                  <InputText
                    label='Token URL'
                    {...field}
                  />
                )}
              />

              {providerType === 'oidc' && (
                <FormField
                  control={form.control}
                  name='userinfoUrl'
                  render={({ field }) => (
                    <InputText
                      label='User Info URL'
                      {...field}
                    />
                  )}
                />
              )}
            </div>
          </BlockContent>
        )}

        {providerType === 'saml' && (
          <BlockContent title='SAML Settings'>
            <div className='flex flex-col gap-5'>
              <FormField
                control={form.control}
                name='entityId'
                render={({ field }) => (
                  <InputText
                    label='Entity ID'
                    {...field}
                  />
                )}
              />

              <FormField
                control={form.control}
                name='ssoUrl'
                render={({ field }) => (
                  <InputText
                    label='Single Sign-On URL'
                    {...field}
                  />
                )}
              />
            </div>
          </BlockContent>
        )}

        {providerType === 'ldap' && (
          <BlockContent title='LDAP Settings'>
            <div className='flex flex-col gap-5'>
              <FormField
                control={form.control}
                name='ldapUrl'
                render={({ field }) => (
                  <InputText
                    label='LDAP URL'
                    {...field}
                  />
                )}
              />

              <FormField
                control={form.control}
                name='bindDn'
                render={({ field }) => (
                  <InputText
                    label='Bind DN'
                    {...field}
                  />
                )}
              />

              <FormField
                control={form.control}
                name='bindCredential'
                render={({ field }) => (
                  <InputText
                    label='Bind Credential'
                    type='password'
                    {...field}
                  />
                )}
              />
            </div>
          </BlockContent>
        )}
      </div>

      <FloatingActionBar
        show={formIsValid ?? false}
        title='Create Provider'
        onCancel={handleBack}
        description='Create a new identity provider with the specified configuration.'
        actions={[
          {
            label: 'Create',
            onClick: handleSubmit,
          }
        ]}
      />
    </div>
  )
}
