import { Form, FormControl, FormField, FormItem, FormLabel, FormMessage } from '@/components/ui/form'
import { Input } from '@/components/ui/input'
import { Button } from '@/components/ui/button'
import { ArrowLeft } from 'lucide-react'
import { UseFormReturn } from 'react-hook-form'
import { MapperTemplate } from '../constants/protocol-mapper-templates'
import { MapperConfigFields } from '../components/mapper-config-fields'
import { MapperTemplateFormSchema } from '../schemas/mapper-template-form.schema'

interface PageCreateProtocolMapperProps {
  template: MapperTemplate
  form: UseFormReturn<MapperTemplateFormSchema>
  configValues: Record<string, string>
  onConfigChange: (key: string, value: string) => void
  isValid: boolean
  isPending: boolean
  onSubmit: () => void
  onCancel: () => void
}

export default function PageCreateProtocolMapper({
  template,
  form,
  configValues,
  onConfigChange,
  isValid,
  isPending,
  onSubmit,
  onCancel,
}: PageCreateProtocolMapperProps) {
  return (
    <Form {...form}>
      <div className='flex flex-col gap-8 max-w-2xl'>

        {/* Back + template badge */}
        <div className='flex flex-col gap-4'>
          <button
            type='button'
            onClick={onCancel}
            className='flex items-center gap-1.5 text-sm text-muted-foreground hover:text-foreground transition-colors w-fit'
          >
            <ArrowLeft className='h-3.5 w-3.5' />
            Protocol Mappers
          </button>

          <div className='flex items-start gap-4 rounded-lg border border-border bg-muted/30 p-4'>
            <span className='text-3xl leading-none'>{template.icon}</span>
            <div className='flex flex-col gap-0.5'>
              <p className='text-base font-semibold'>{template.name}</p>
              <p className='text-sm text-muted-foreground'>{template.description}</p>
              {!template.isCustom && (
                <p className='mt-1 text-xs font-mono text-muted-foreground/60'>
                  {template.mapper_type}
                </p>
              )}
            </div>
          </div>
        </div>

        {/* Mapper name */}
        <section className='flex flex-col gap-4'>
          <div>
            <p className='text-xs text-muted-foreground mb-0.5'>General</p>
            <h2 className='text-base font-semibold'>Identity</h2>
          </div>
          <FormField
            control={form.control}
            name='name'
            render={({ field }) => (
              <FormItem>
                <FormLabel>Name</FormLabel>
                <FormControl>
                  <Input placeholder='Mapper name' {...field} />
                </FormControl>
                <FormMessage />
              </FormItem>
            )}
          />
          {template.isCustom && (
            <FormField
              control={form.control}
              name='mapper_type'
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Mapper Type</FormLabel>
                  <FormControl>
                    <Input placeholder='e.g. oidc-usermodel-property-mapper' {...field} />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />
          )}
        </section>

        {/* Dynamic config fields */}
        {(template.fields.length > 0 || template.isCustom) && (
          <section className='flex flex-col gap-4'>
            <div>
              <p className='text-xs text-muted-foreground mb-0.5'>Mapper configuration</p>
              <h2 className='text-base font-semibold'>Settings</h2>
            </div>

            {template.isCustom ? (
              <FormField
                control={form.control}
                name='config_json'
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>Config (JSON)</FormLabel>
                    <FormControl>
                      <textarea
                        placeholder='{"key": "value"}'
                        rows={8}
                        className='flex w-full rounded-md border border-input bg-background px-3 py-2 text-sm font-mono shadow-sm placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50 resize-none'
                        {...field}
                      />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
            ) : (
              <MapperConfigFields
                fields={template.fields}
                values={configValues}
                onChange={onConfigChange}
              />
            )}
          </section>
        )}

        {/* Actions */}
        <div className='flex items-center gap-3 pt-2 border-t'>
          <Button
            type='button'
            onClick={onSubmit}
            disabled={!isValid || isPending}
          >
            {isPending ? 'Creating...' : 'Create mapper'}
          </Button>
          <Button type='button' variant='outline' onClick={onCancel}>
            Cancel
          </Button>
        </div>

      </div>
    </Form>
  )
}
