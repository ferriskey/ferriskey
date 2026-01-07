import { UseFormReturn } from 'react-hook-form'
import { CreateProviderSchema } from '../schemas/create-provider.schema'
import { Button } from '@/components/ui/button'
import { ArrowLeft, CheckCircle } from 'lucide-react'
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

export interface PageCreateProviderProps {
  form: UseFormReturn<CreateProviderSchema>
  handleBack: () => void
  handleSubmit: () => void
  formIsValid?: boolean
}

export default function PageCreateProvider({
  form,
  handleBack,
  handleSubmit,
  formIsValid
}: PageCreateProviderProps) {
  return (
    <div className='flex flex-col p-4 gap-4'>
      <div className='flex items-center gap-3'>
        <Button variant='ghost' size='icon' onClick={handleBack}>
          <ArrowLeft className='h-3 w-3' />
        </Button>
        <span className='text-gray-500 text-sm font-medium'>Back to providers</span>
      </div>

      <div className='flex flex-col mb-4'>
        <Heading size={3} className='text-foreground'>
          Create Federation Provider
        </Heading>
        <p className='text-sm text-muted-foreground mt-1'>
          Configure a new external user storage provider for federated authentication.
        </p>
      </div>

      <BlockContent title='Basic Settings' customWidth='lg:w-2/3'>
        <div className='flex flex-col gap-5'>
          <FormField
            control={form.control}
            name='name'
            render={({ field }) => (
              <InputText
                label='Provider Name'
                placeholder='e.g., Corporate LDAP'
                error={form.formState.errors.name?.message}
                {...field}
              />
            )}
          />

          <div className='grid grid-cols-2 gap-4'>
            <FormField
              control={form.control}
              name='type'
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Provider Type</FormLabel>
                  <Select onValueChange={field.onChange} defaultValue={field.value}>
                    <FormControl>
                      <SelectTrigger>
                        <SelectValue placeholder='Select provider type' />
                      </SelectTrigger>
                    </FormControl>
                    <SelectContent>
                      <SelectItem value='LDAP'>LDAP</SelectItem>
                      <SelectItem value='ActiveDirectory'>Active Directory</SelectItem>
                      <SelectItem value='Kerberos'>Kerberos</SelectItem>
                      <SelectItem value='Custom'>Custom</SelectItem>
                    </SelectContent>
                  </Select>
                </FormItem>
              )}
            />

            <FormField
              control={form.control}
              name='priority'
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Priority</FormLabel>
                  <Select onValueChange={field.onChange} defaultValue={field.value}>
                    <FormControl>
                      <SelectTrigger>
                        <SelectValue placeholder='Select priority' />
                      </SelectTrigger>
                    </FormControl>
                    <SelectContent>
                      <SelectItem value='Primary'>Primary</SelectItem>
                      <SelectItem value='Secondary'>Secondary</SelectItem>
                      <SelectItem value='Development'>Development</SelectItem>
                      <SelectItem value='Legacy'>Legacy</SelectItem>
                      <SelectItem value='Custom'>Custom</SelectItem>
                    </SelectContent>
                  </Select>
                </FormItem>
              )}
            />
          </div>

          <FormField
            control={form.control}
            name='enabled'
            render={({ field }) => (
              <FormItem className='flex flex-row items-center justify-between gap-5 rounded-lg border p-3 shadow-sm'>
                <div className='space-y-0.5'>
                  <FormLabel>Enable Provider</FormLabel>
                  <FormDescription>
                    Toggle to enable or disable the provider. Disabled providers will not sync users.
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

      <BlockContent title='Connection Settings' customWidth='lg:w-2/3'>
        <div className='flex flex-col gap-5'>
          <FormField
            control={form.control}
            name='connectionUrl'
            render={({ field }) => (
              <InputText
                label='Connection URL'
                placeholder='ldap://ldap.company.local:389 or ldaps://ldap.company.local:636'
                error={form.formState.errors.connectionUrl?.message}
                {...field}
              />
            )}
          />

          <FormField
            control={form.control}
            name='baseDn'
            render={({ field }) => (
              <InputText
                label='Base DN'
                placeholder='dc=company,dc=local'
                error={form.formState.errors.baseDn?.message}
                {...field}
              />
            )}
          />

          <div className='grid grid-cols-2 gap-4'>
            <FormField
              control={form.control}
              name='bindDn'
              render={({ field }) => (
                <InputText
                  label='Bind DN'
                  placeholder='cn=admin,dc=company,dc=local'
                  error={form.formState.errors.bindDn?.message}
                  {...field}
                />
              )}
            />

            <FormField
              control={form.control}
              name='bindPassword'
              render={({ field }) => (
                <InputText
                  label='Bind Password'
                  type='password'
                  placeholder='••••••••'
                  error={form.formState.errors.bindPassword?.message}
                  {...field}
                />
              )}
            />
          </div>
        </div>
      </BlockContent>

      <BlockContent title='User Search Settings' customWidth='lg:w-2/3'>
        <div className='flex flex-col gap-5'>
          <FormField
            control={form.control}
            name='userSearchBase'
            render={({ field }) => (
              <InputText
                label='User Search Base'
                placeholder='ou=users,dc=company,dc=local'
                error={form.formState.errors.userSearchBase?.message}
                {...field}
              />
            )}
          />

          <FormField
            control={form.control}
            name='userSearchFilter'
            render={({ field }) => (
              <InputText
                label='User Search Filter'
                placeholder='(objectClass=person)'
                error={form.formState.errors.userSearchFilter?.message}
                {...field}
              />
            )}
          />
        </div>
      </BlockContent>

      <BlockContent title='Synchronization Settings' customWidth='lg:w-2/3'>
        <div className='flex flex-col gap-5'>
          <FormField
            control={form.control}
            name='syncInterval'
            render={({ field }) => (
              <InputText
                label='Sync Interval (seconds)'
                type='number'
                placeholder='3600'
                error={form.formState.errors.syncInterval?.message}
                {...field}
                onChange={(e) => field.onChange(parseInt(e.target.value))}
              />
            )}
          />
        </div>
      </BlockContent>

      <FloatingActionBar
        show={formIsValid ?? false}
        title='Create Provider'
        description='Once created, the provider will be available for user synchronization.'
        icon={<CheckCircle className='h-4 w-4' />}
        onCancel={handleBack}
        actions={[
          {
            label: 'Create Provider',
            onClick: handleSubmit,
          }
        ]}
      />
    </div>
  )
}
