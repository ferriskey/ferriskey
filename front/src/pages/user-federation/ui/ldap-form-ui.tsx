import { UseFormReturn } from 'react-hook-form'
import { CreateLdapProviderSchema } from '../schemas/ldap-provider.schema'
import { Button } from '@/components/ui/button'
import { ArrowLeft, CheckCircle, Server, Key } from 'lucide-react'
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
import { RadioGroup, RadioGroupItem } from '@/components/ui/radio-group'
import { Label } from '@/components/ui/label'
import { Badge } from '@/components/ui/badge'
import { cn } from '@/lib/utils'

interface LdapFormUiProps {
  form: UseFormReturn<CreateLdapProviderSchema>
  handleBack: () => void
  handleSubmit: () => void
  onTypeChange: (type: 'LDAP' | 'Kerberos') => void
  isEditMode?: boolean
}

export default function LdapFormUi({
  form,
  handleBack,
  handleSubmit,
  onTypeChange,
  isEditMode = false,
}: LdapFormUiProps) {
  const providerTypes = [
    {
      value: 'LDAP' as const,
      label: 'LDAP',
      description: 'Connect to LDAP directory for user authentication',
      icon: Server,
      disabled: false,
    },
    {
      value: 'Kerberos' as const,
      label: 'Kerberos',
      description: 'Integrate with Kerberos for network authentication',
      icon: Key,
      disabled: true,
    },
  ]

  return (
    <div className='flex flex-col p-4 gap-4'>
      {/* Header */}
      <div className='flex items-center gap-3'>
        <Button variant='ghost' size='icon' onClick={handleBack} type='button'>
          <ArrowLeft className='h-4 w-4' />
        </Button>
        <div>
          <Heading size={3} className='text-foreground'>
            {isEditMode ? 'Edit User Federation Provider' : 'Add User Federation Provider'}
          </Heading>
          <p className='text-sm text-muted-foreground mt-1'>
            {isEditMode
              ? 'Update the configuration of your external user storage provider.'
              : 'Configure a new external user storage provider.'}
          </p>
        </div>
      </div>

      {/* Provider Type Selection - Only show in create mode */}
      {!isEditMode && (
        <BlockContent title='Select Provider Type' customWidth='lg:w-2/3'>
          <RadioGroup value='LDAP' onValueChange={(value) => onTypeChange(value as 'LDAP' | 'Kerberos')}>
            <div className='grid grid-cols-1 md:grid-cols-2 gap-4'>
              {providerTypes.map((type) => {
                const Icon = type.icon
                const isSelected = type.value === 'LDAP'

                return (
                  <div key={type.value} className='relative'>
                    <RadioGroupItem
                      value={type.value}
                      id={type.value}
                      className='peer sr-only'
                      disabled={type.disabled}
                    />
                    <Label
                      htmlFor={type.value}
                      className={cn(
                        'flex flex-col p-4 rounded-lg border-2 transition-all',
                        type.disabled
                          ? 'cursor-not-allowed opacity-60'
                          : 'cursor-pointer hover:border-primary/50 hover:shadow-sm',
                        isSelected
                          ? 'border-primary bg-primary/5'
                          : 'border-border'
                      )}
                    >
                      <div className='flex items-start gap-3 mb-3'>
                        <div className={cn(
                          'flex h-10 w-10 items-center justify-center rounded-lg shrink-0',
                          type.disabled
                            ? 'bg-muted'
                            : isSelected ? 'bg-primary/20' : 'bg-muted'
                        )}>
                          <Icon className={cn(
                            'h-5 w-5',
                            type.disabled
                              ? 'text-muted-foreground'
                              : isSelected ? 'text-primary' : 'text-muted-foreground'
                          )} />
                        </div>
                        <div className='flex-1'>
                          <div className='flex items-center gap-2 mb-1'>
                            <h4 className='font-semibold'>{type.label}</h4>
                            {type.disabled && (
                              <Badge variant='secondary' className='text-xs'>
                                Coming Soon
                              </Badge>
                            )}
                          </div>
                          <p className='text-sm text-muted-foreground'>
                            {type.description}
                          </p>
                        </div>
                        {isSelected && !type.disabled && (
                          <div className='h-5 w-5 rounded-full bg-primary flex items-center justify-center'>
                            <CheckCircle className='h-3 w-3 text-primary-foreground' />
                          </div>
                        )}
                      </div>
                    </Label>
                  </div>
                )
              })}
            </div>
          </RadioGroup>
        </BlockContent>
      )}

      {/* Basic Settings */}
      <BlockContent title='Basic Settings' customWidth='lg:w-2/3'>
        <div className='flex flex-col gap-5'>
          <FormField
            control={form.control}
            name='name'
            render={({ field }) => (
              <InputText
                label='Provider Name'
                error={form.formState.errors.name?.message}
                {...field}
              />
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
                    <SelectTrigger className='bg-white'>
                      <SelectValue placeholder='Select priority' />
                    </SelectTrigger>
                  </FormControl>
                  <SelectContent>
                    <SelectItem value='Primary'>Primary</SelectItem>
                    <SelectItem value='Secondary'>Secondary</SelectItem>
                    <SelectItem value='Development'>Development</SelectItem>
                    <SelectItem value='Legacy'>Legacy</SelectItem>
                  </SelectContent>
                </Select>
              </FormItem>
            )}
          />

          <FormField
            control={form.control}
            name='enabled'
            render={({ field }) => (
              <FormItem className='flex flex-row items-center justify-between gap-5 rounded-md border p-3  bg-white'>
                <div className='space-y-0.5'>
                  <FormLabel>Enable Provider</FormLabel>
                  <FormDescription>
                    Toggle to enable or disable the provider.
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

      {/* Connection Settings */}
      <BlockContent title='Connection Settings' customWidth='lg:w-2/3'>
        <div className='flex flex-col gap-5'>
          <FormField
            control={form.control}
            name='connectionUrl'
            render={({ field }) => (
              <InputText
                label='Connection URL'
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
                  error={form.formState.errors.bindPassword?.message}
                  {...field}
                />
              )}
            />
          </div>

          <FormField
            control={form.control}
            name='useTls'
            render={({ field }) => (
              <FormItem className='flex flex-row items-center justify-between gap-5 rounded-md border p-3 bg-white'>
                <div className='space-y-0.5'>
                  <FormLabel>Use TLS</FormLabel>
                  <FormDescription>
                    Enable TLS/SSL encryption for LDAP connections
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

      {/* User Search Settings */}
      <BlockContent title='User Search Settings' customWidth='lg:w-2/3'>
        <div className='flex flex-col gap-5'>
          <FormField
            control={form.control}
            name='userSearchFilter'
            render={({ field }) => (
              <InputText
                label='User Search Filter'
                error={form.formState.errors.userSearchFilter?.message}
                {...field}
              />
            )}
          />
        </div>
      </BlockContent>

      {/* Synchronization Settings */}
      <BlockContent title='Synchronization Settings' customWidth='lg:w-2/3'>
        <div className='flex flex-col gap-5'>
          <FormField
            control={form.control}
            name='syncInterval'
            render={({ field }) => (
              <InputText
                label='Sync Interval (seconds)'
                type='number'
                error={form.formState.errors.syncInterval?.message}
                {...field}
              />
            )}
          />
        </div>
      </BlockContent>

      <FloatingActionBar
        show={form.formState.isValid}
        title={isEditMode ? 'Update Provider' : 'Create Provider'}
        description={isEditMode
          ? 'Update the provider configuration to apply changes.'
          : 'Once created, the provider will be available for user synchronization.'}
        icon={<CheckCircle className='h-4 w-4' />}
        onCancel={handleBack}
        actions={[
          {
            label: isEditMode ? 'Update Provider' : 'Create Provider',
            onClick: handleSubmit,
          }
        ]}
      />
    </div>
  )
}
