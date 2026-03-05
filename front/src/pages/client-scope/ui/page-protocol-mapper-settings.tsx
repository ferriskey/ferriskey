import {
  Form,
  FormControl,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from '@/components/ui/form'
import { Input } from '@/components/ui/input'
import { ArrowLeft } from 'lucide-react'
import { UseFormReturn } from 'react-hook-form'
import FloatingActionBar from '@/components/ui/floating-action-bar'
import BlockContent from '@/components/ui/block-content'
import { MapperConfigFields } from '../components/mapper-config-fields'
import { MapperTemplate } from '../constants/protocol-mapper-templates'
import { MapperSettingsSchema } from '../schemas/mapper-settings.schema'
import { Schemas } from '@/api/api.client'

import ProtocolMapper = Schemas.ProtocolMapper

// ─── Types ────────────────────────────────────────────────────────────────────

export interface PageProtocolMapperSettingsProps {
  mapper: ProtocolMapper
  template: MapperTemplate | null
  form: UseFormReturn<MapperSettingsSchema>
  configValues: Record<string, string>
  onConfigChange: (key: string, value: string) => void
  hasChanges: boolean
  isPending: boolean
  onSubmit: () => void
  onCancel: () => void
  onReset: () => void
}

// ─── Component ────────────────────────────────────────────────────────────────

export default function PageProtocolMapperSettings({
  mapper,
  template,
  form,
  configValues,
  onConfigChange,
  hasChanges,
  isPending,
  onSubmit,
  onCancel,
  onReset,
}: PageProtocolMapperSettingsProps) {
  return (
    <Form {...form}>
      <div className='flex flex-col gap-8 max-w-2xl'>

        {/* Back + mapper info banner */}
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
            <span className='text-3xl leading-none'>{template?.icon ?? '⚙️'}</span>
            <div className='flex flex-col gap-0.5'>
              <p className='text-base font-semibold'>{template?.name ?? mapper.name}</p>
              {template?.description && (
                <p className='text-sm text-muted-foreground'>{template.description}</p>
              )}
              <p className='mt-1 text-xs font-mono text-muted-foreground/60'>
                {mapper.mapper_type}
              </p>
            </div>
          </div>
        </div>

        {/* Read-only info cards */}
        <div className='grid grid-cols-1 md:grid-cols-2 gap-4'>
          <div className='border p-4 rounded-sm flex flex-col gap-2'>
            <span className='text-xs text-muted-foreground'>Mapper type</span>
            <span className='text-sm font-mono text-foreground'>{mapper.mapper_type}</span>
          </div>
          <div className='border p-4 rounded-sm flex flex-col gap-2'>
            <span className='text-xs text-muted-foreground'>Created at</span>
            <span className='text-sm text-foreground'>
              {new Date(mapper.created_at).toLocaleDateString('fr-FR')}
            </span>
          </div>
        </div>

        {/* General section */}
        <BlockContent title='General'>
          <div className='flex flex-col gap-3'>
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
          </div>
        </BlockContent>

        {/* Dynamic config fields (template match) */}
        {template && template.fields.length > 0 && (
          <BlockContent title='Settings'>
            <MapperConfigFields
              fields={template.fields}
              values={configValues}
              onChange={onConfigChange}
            />
          </BlockContent>
        )}

        {/* Fallback: raw JSON for unrecognised mappers */}
        {!template && (
          <BlockContent title='Configuration (JSON)'>
            <FormField
              control={form.control}
              name='config_json'
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Config</FormLabel>
                  <FormControl>
                    <textarea
                      rows={10}
                      className='flex w-full rounded-md border border-input bg-background px-3 py-2 text-sm font-mono shadow-sm placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50 resize-none'
                      {...field}
                    />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />
          </BlockContent>
        )}

      </div>

      {/* Floating save bar */}
      <FloatingActionBar
        show={hasChanges}
        title='Unsaved changes'
        description='You have unsaved changes. Do you want to save them?'
        actions={[
          {
            label: isPending ? 'Saving…' : 'Save',
            variant: 'default',
            onClick: onSubmit,
          },
        ]}
        onCancel={onReset}
      />
    </Form>
  )
}
