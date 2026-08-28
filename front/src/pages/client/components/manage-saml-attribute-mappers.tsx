import { Button } from '@/components/ui/button'
import { ConfirmDeleteAlert } from '@/components/confirm-delete-alert'
import { Form, FormField, FormItem, FormLabel, FormControl } from '@/components/ui/form'
import { InputText } from '@/components/ui/input-text'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { useConfirmDeleteAlert } from '@/hooks/use-confirm-delete-alert.ts'
import {
  ATTRIBUTE_NAME_FORMAT_OPTIONS,
  AttributeMapperDraft,
  BUILT_IN_SOURCE_OPTIONS,
  COMMON_PROFILE_MAPPERS,
  CUSTOM_ATTRIBUTE_SOURCE,
  DEFAULT_ATTRIBUTE_NAME_FORMAT,
  describeAttributeNameFormat,
  describeAttributeSource,
  toCustomAttributeSource,
} from '@/lib/saml'
import type { Schemas } from '@/api/api.client'
import { zodResolver } from '@hookform/resolvers/zod'
import { Trash2 } from 'lucide-react'
import { useForm, useWatch } from 'react-hook-form'
import {
  samlAttributeMapperSchema,
  SamlAttributeMapperSchema,
} from '../schemas/saml-service-provider.schema'
import { SamlAttributeMapperInput } from '@/hooks/use-saml-service-provider'

export interface ManageSamlAttributeMappersProps {
  mappers: Schemas.SamlAttributeMapper[]
  isCreating: boolean
  isDeleting: boolean
  onAdd: (input: SamlAttributeMapperInput) => Promise<boolean>
  onDelete: (mapperId: string) => Promise<boolean>
  onAddCommonProfile: (drafts: AttributeMapperDraft[]) => Promise<void>
}

export default function ManageSamlAttributeMappers({
  mappers,
  isCreating,
  isDeleting,
  onAdd,
  onDelete,
  onAddCommonProfile,
}: ManageSamlAttributeMappersProps) {
  const { confirm, ask, close } = useConfirmDeleteAlert()

  const form = useForm<SamlAttributeMapperSchema>({
    resolver: zodResolver(samlAttributeMapperSchema),
    defaultValues: {
      name: '',
      source: BUILT_IN_SOURCE_OPTIONS[0].value,
      customKey: '',
      nameFormat: DEFAULT_ATTRIBUTE_NAME_FORMAT,
    },
  })

  const source = useWatch({ control: form.control, name: 'source' })
  const isCustomSource = source === CUSTOM_ATTRIBUTE_SOURCE

  const handleDelete = (mapper: Schemas.SamlAttributeMapper) => {
    ask({
      title: `Stop sending ${mapper.name}?`,
      description:
        'The application will no longer receive this attribute on the next sign-in. Anything relying on it may break.',
      onConfirm: async () => {
        if (await onDelete(mapper.id)) {
          close()
        }
      },
    })
  }

  const onSubmit = async (values: SamlAttributeMapperSchema) => {
    const added = await onAdd({
      name: values.name,
      source: isCustomSource ? toCustomAttributeSource(values.customKey) : values.source,
      nameFormat: values.nameFormat,
    })

    if (added) {
      form.reset({
        name: '',
        source: values.source,
        customKey: '',
        nameFormat: values.nameFormat,
      })
    }
  }

  return (
    <>
      <div className='flex flex-col gap-4'>
        {mappers.length === 0 && (
          <div className='flex flex-col gap-3 rounded-md border border-dashed p-4'>
            <p className='text-sm text-muted-foreground'>
              No attributes are sent yet. Most applications need at least an email address.
            </p>
            <Button
              type='button'
              variant='outline'
              className='self-start'
              disabled={isCreating}
              onClick={() => void onAddCommonProfile(COMMON_PROFILE_MAPPERS)}
            >
              Add email, first_name and last_name
            </Button>
          </div>
        )}

        {mappers.map((mapper) => (
          <div key={mapper.id} className='flex items-center gap-3 rounded-md border px-3 py-2'>
            <span className='text-sm font-mono truncate'>{mapper.name}</span>
            <span className='text-xs text-muted-foreground shrink-0'>from</span>
            <span className='flex-1 text-sm truncate'>
              {describeAttributeSource(mapper.source)}
            </span>
            <span className='text-xs text-muted-foreground shrink-0'>
              {describeAttributeNameFormat(mapper.name_format)}
            </span>
            <Button
              className='text-red-500 shrink-0'
              variant='ghost'
              size='icon'
              disabled={isDeleting}
              aria-label={`Remove the ${mapper.name} attribute mapping`}
              onClick={() => handleDelete(mapper)}
            >
              <Trash2 size={14} />
            </Button>
          </div>
        ))}

        <Form {...form}>
          <form onSubmit={form.handleSubmit(onSubmit)} className='flex flex-col gap-4'>
            <FormField
              control={form.control}
              name='name'
              render={({ field }) => (
                <InputText
                  {...field}
                  label='Attribute name'
                  error={form.formState.errors.name?.message}
                />
              )}
            />

            <FormField
              control={form.control}
              name='source'
              render={({ field }) => (
                <FormItem>
                  <FormLabel>User detail to send</FormLabel>
                  <Select onValueChange={field.onChange} value={field.value}>
                    <FormControl>
                      <SelectTrigger>
                        <SelectValue placeholder='Select a user detail' />
                      </SelectTrigger>
                    </FormControl>
                    <SelectContent>
                      {BUILT_IN_SOURCE_OPTIONS.map((option) => (
                        <SelectItem key={option.value} value={option.value}>
                          {option.label}
                        </SelectItem>
                      ))}
                      <SelectItem value={CUSTOM_ATTRIBUTE_SOURCE}>
                        Custom user attribute
                      </SelectItem>
                    </SelectContent>
                  </Select>
                </FormItem>
              )}
            />

            {isCustomSource && (
              <FormField
                control={form.control}
                name='customKey'
                render={({ field }) => (
                  <InputText
                    {...field}
                    label='Attribute key'
                    error={form.formState.errors.customKey?.message}
                  />
                )}
              />
            )}

            <FormField
              control={form.control}
              name='nameFormat'
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Name format</FormLabel>
                  <Select onValueChange={field.onChange} value={field.value}>
                    <FormControl>
                      <SelectTrigger>
                        <SelectValue placeholder='Select a name format' />
                      </SelectTrigger>
                    </FormControl>
                    <SelectContent>
                      {ATTRIBUTE_NAME_FORMAT_OPTIONS.map((option) => (
                        <SelectItem key={option.value} value={option.value}>
                          {option.label}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </FormItem>
              )}
            />

            <Button type='submit' disabled={isCreating}>
              Add attribute
            </Button>
          </form>
        </Form>
      </div>

      <ConfirmDeleteAlert
        title={confirm.title}
        description={confirm.description}
        open={confirm.open}
        onConfirm={confirm.onConfirm}
        onCancel={close}
      />
    </>
  )
}
