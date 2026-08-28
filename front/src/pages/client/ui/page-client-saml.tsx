import { InputText } from '@/components/ui/input-text'
import { FormControl, FormField, FormItem, FormLabel } from '@/components/ui/form'
import { Switch } from '@/components/ui/switch'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import FloatingActionBar from '@/components/ui/floating-action-bar'
import { Button } from '@/components/ui/button'
import { NAME_ID_FORMAT_OPTIONS } from '@/lib/saml'
import { UseFormReturn } from 'react-hook-form'
import { SamlServiceProviderSchema } from '../schemas/saml-service-provider.schema'
import ManageSamlAttributeMappers, {
  ManageSamlAttributeMappersProps,
} from '../components/manage-saml-attribute-mappers'

export interface PageClientSamlProps {
  form: UseFormReturn<SamlServiceProviderSchema>
  isConfigured: boolean
  isSaving: boolean
  hasChanges: boolean
  handleSubmit: () => void
  mapperProps: ManageSamlAttributeMappersProps
}

export default function PageClientSaml({
  form,
  isConfigured,
  isSaving,
  hasChanges,
  handleSubmit,
  mapperProps,
}: PageClientSamlProps) {
  return (
    <div className='flex flex-col gap-8'>
      {!isConfigured && (
        <div className='rounded-md border bg-muted/40 p-4'>
          <p className='text-sm font-medium'>This client does not use SAML yet</p>
          <p className='text-sm text-muted-foreground mt-1'>
            Fill in the two values the application shows on its SAML settings page, then save.
            FerrisKey will start answering SAML sign-in requests for it.
          </p>
        </div>
      )}

      <div className='flex flex-col gap-1'>
        <div className='mb-4'>
          <p className='text-xs text-muted-foreground mb-0.5'>SAML 2.0</p>
          <h2 className='text-base font-semibold'>Service Provider</h2>
          <p className='text-sm text-muted-foreground mt-1'>
            Identifies the application and tells FerrisKey where to send the assertion.
          </p>
        </div>

        <FormField
          control={form.control}
          name='spEntityId'
          render={({ field }) => (
            <div className='flex items-start justify-between py-4 border-t'>
              <div className='w-1/3'>
                <p className='text-sm font-medium'>Entity ID</p>
                <p className='text-sm text-muted-foreground mt-0.5'>
                  The unique identifier the application publishes for itself, for example{' '}
                  <code>https://chat.acme.com/saml/sp/1</code>.
                </p>
              </div>
              <div className='w-1/2'>
                <InputText
                  label='Entity ID'
                  name='sp_entity_id'
                  value={field.value}
                  onChange={field.onChange}
                  onBlur={field.onBlur}
                  error={form.formState.errors.spEntityId?.message}
                />
              </div>
            </div>
          )}
        />

        <FormField
          control={form.control}
          name='acsUrl'
          render={({ field }) => (
            <div className='flex items-start justify-between py-4 border-t'>
              <div className='w-1/3'>
                <p className='text-sm font-medium'>Assertion Consumer Service URL</p>
                <p className='text-sm text-muted-foreground mt-0.5'>
                  Where the signed assertion is posted after the user signs in.
                </p>
              </div>
              <div className='w-1/2'>
                <InputText
                  label='ACS URL'
                  name='acs_url'
                  value={field.value}
                  onChange={field.onChange}
                  onBlur={field.onBlur}
                  error={form.formState.errors.acsUrl?.message}
                />
              </div>
            </div>
          )}
        />

        <FormField
          control={form.control}
          name='nameIdFormat'
          render={({ field }) => (
            <div className='flex items-start justify-between py-4 border-t'>
              <div className='w-1/3'>
                <p className='text-sm font-medium'>Name ID Format</p>
                <p className='text-sm text-muted-foreground mt-0.5'>
                  How the user is identified inside the assertion.
                </p>
              </div>
              <div className='w-1/2'>
                <FormItem>
                  <Select onValueChange={field.onChange} value={field.value}>
                    <FormControl>
                      <SelectTrigger>
                        <SelectValue placeholder='Select a name ID format' />
                      </SelectTrigger>
                    </FormControl>
                    <SelectContent>
                      {NAME_ID_FORMAT_OPTIONS.map((option) => (
                        <SelectItem key={option.value} value={option.value}>
                          {option.label}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </FormItem>
              </div>
            </div>
          )}
        />
      </div>

      <div className='flex flex-col gap-1'>
        <div className='mb-4'>
          <p className='text-xs text-muted-foreground mb-0.5'>Assertion security</p>
          <h2 className='text-base font-semibold'>Signatures</h2>
        </div>

        <SignatureToggle
          form={form}
          name='signAssertions'
          title='Sign Assertions'
          description='Sign the assertion itself. Almost every application expects this.'
        />

        <SignatureToggle
          form={form}
          name='signDocuments'
          title='Sign Documents'
          description='Sign the whole SAML response in addition to the assertion.'
        />

        <SignatureToggle
          form={form}
          name='wantAuthnRequestsSigned'
          title='Require Signed Authentication Requests'
          description='Reject sign-in requests from this application unless they carry a valid signature.'
        />
      </div>

      {!isConfigured && (
        <div>
          <Button type='button' disabled={isSaving} onClick={handleSubmit}>
            Enable SAML
          </Button>
        </div>
      )}

      {isConfigured && (
        <div className='flex flex-col gap-1'>
          <div className='mb-4'>
            <p className='text-xs text-muted-foreground mb-0.5'>Sent with every assertion</p>
            <h2 className='text-base font-semibold'>Attribute Mappings</h2>
            <p className='text-sm text-muted-foreground mt-1'>
              User details sent alongside the assertion. Match the attribute names the application
              asks for.
            </p>
          </div>

          <div className='py-4 border-t'>
            <ManageSamlAttributeMappers {...mapperProps} />
          </div>
        </div>
      )}

      {isConfigured && (
        <FloatingActionBar
          show={hasChanges}
          title='Save Changes'
          actions={[
            {
              label: 'Save',
              variant: 'default',
              onClick: handleSubmit,
            },
          ]}
          description='Save the SAML service provider settings for this client.'
          onCancel={() => form.reset()}
        />
      )}
    </div>
  )
}

interface SignatureToggleProps {
  form: UseFormReturn<SamlServiceProviderSchema>
  name: 'signAssertions' | 'signDocuments' | 'wantAuthnRequestsSigned'
  title: string
  description: string
}

function SignatureToggle({ form, name, title, description }: SignatureToggleProps) {
  return (
    <FormField
      control={form.control}
      name={name}
      render={({ field }) => (
        <div className='flex items-center justify-between py-4 border-t'>
          <div className='w-1/3'>
            <p className='text-sm font-medium'>{title}</p>
            <p className='text-sm text-muted-foreground mt-0.5'>{description}</p>
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
  )
}
