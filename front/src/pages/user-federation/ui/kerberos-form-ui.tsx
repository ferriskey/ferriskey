import { UseFormReturn } from 'react-hook-form'
import { CreateKerberosProviderSchema } from '../schemas/kerberos-provider.schema'
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

interface KerberosFormUiProps {
  form: UseFormReturn<CreateKerberosProviderSchema>
  handleBack: () => void
  handleSubmit: () => void
  onTypeChange: (type: 'LDAP' | 'Kerberos') => void
}

export default function KerberosFormUi({
  form,
  handleBack,
  handleSubmit,
  onTypeChange,
}: KerberosFormUiProps) {
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
      disabled: false,
    },
  ]

  return (
    <div className='flex flex-col p-4 gap-4'>
      {/* Header */}
      <div className='flex items-center gap-3'>
        <Button variant='ghost' size='icon' onClick={handleBack} type='button'>
          <ArrowLeft className='h-3 w-3' />
        </Button>
        <div>
          <Heading size={3} className='text-foreground'>
            Add User Federation Provider
          </Heading>
          <p className='text-sm text-muted-foreground mt-1'>
            Configure a new external user storage provider.
          </p>
        </div>
      </div>

      {/* Provider Type Selection */}
      <BlockContent title='Select Provider Type' customWidth='lg:w-2/3'>
        <RadioGroup value='Kerberos' onValueChange={(value) => onTypeChange(value as 'LDAP' | 'Kerberos')}>
          <div className='grid grid-cols-1 md:grid-cols-2 gap-4'>
            {providerTypes.map((type) => {
              const Icon = type.icon
              const isSelected = type.value === 'Kerberos'

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
                    <SelectTrigger>
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
              <FormItem className='flex flex-row items-center justify-between gap-5 rounded-lg border p-3 shadow-sm'>
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

      {/* Kerberos Configuration */}
      <BlockContent title='Kerberos Configuration' customWidth='lg:w-2/3'>
        <div className='flex flex-col gap-5'>
          <FormField
            control={form.control}
            name='kerberosRealm'
            render={({ field }) => (
              <InputText
                label='Kerberos Realm'
                error={form.formState.errors.kerberosRealm?.message}
                {...field}
              />
            )}
          />

          <FormField
            control={form.control}
            name='kdcServer'
            render={({ field }) => (
              <InputText
                label='KDC Server'
                error={form.formState.errors.kdcServer?.message}
                {...field}
              />
            )}
          />

          <FormField
            control={form.control}
            name='adminServer'
            render={({ field }) => (
              <InputText
                label='Admin Server (optional)'
                error={form.formState.errors.adminServer?.message}
                {...field}
              />
            )}
          />

          <FormField
            control={form.control}
            name='allowPasswordAuth'
            render={({ field }) => (
              <FormItem className='flex flex-row items-center justify-between gap-5 rounded-lg border p-3 shadow-sm'>
                <div className='space-y-0.5'>
                  <FormLabel>Allow Password Authentication</FormLabel>
                  <FormDescription>
                    Enable password-based authentication fallback
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

      <FloatingActionBar
        show={form.formState.isValid}
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
