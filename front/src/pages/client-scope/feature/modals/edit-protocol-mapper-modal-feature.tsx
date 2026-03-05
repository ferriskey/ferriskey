import { Schemas } from '@/api/api.client'
import { useUpdateProtocolMapper } from '@/api/client-scope.api'
import { Form } from '@/components/ui/form'
import { zodResolver } from '@hookform/resolvers/zod'
import { useEffect } from 'react'
import { useForm } from 'react-hook-form'
import { useParams } from 'react-router'
import { RouterParams } from '@/routes/router'
import {
  UpdateProtocolMapperSchema,
  updateProtocolMapperSchema,
} from '../../schemas/update-protocol-mapper.schema'
import EditProtocolMapperModal from '../../ui/modals/edit-protocol-mapper-modal'

import ProtocolMapper = Schemas.ProtocolMapper

interface EditProtocolMapperModalFeatureProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  mapper: ProtocolMapper | null
}

export default function EditProtocolMapperModalFeature({
  open,
  onOpenChange,
  mapper,
}: EditProtocolMapperModalFeatureProps) {
  const { realm_name, scope_id } = useParams<RouterParams>()
  const { mutate: updateMapper, isPending } = useUpdateProtocolMapper()

  const form = useForm<UpdateProtocolMapperSchema>({
    resolver: zodResolver(updateProtocolMapperSchema),
    mode: 'onChange',
    defaultValues: {
      name: '',
      mapper_type: '',
      config: '',
    },
  })

  useEffect(() => {
    if (mapper) {
      form.reset({
        name: mapper.name,
        mapper_type: mapper.mapper_type,
        config: mapper.config ? JSON.stringify(mapper.config, null, 2) : '',
      })
    }
  }, [mapper, form])

  const handleSubmit = form.handleSubmit((values) => {
    if (!realm_name || !scope_id || !mapper) return

    let parsedConfig: unknown = {}
    if (values.config && values.config.trim() !== '') {
      try {
        parsedConfig = JSON.parse(values.config)
      } catch {
        parsedConfig = {}
      }
    }

    updateMapper(
      {
        path: { realm_name, scope_id, mapper_id: mapper.id },
        body: {
          name: values.name ?? null,
          mapper_type: values.mapper_type ?? null,
          config: parsedConfig,
        },
      },
      {
        onSuccess: () => {
          onOpenChange(false)
        },
      }
    )
  })

  const isValid = form.formState.isValid && !isPending

  return (
    <Form {...form}>
      <EditProtocolMapperModal
        open={open}
        onOpenChange={(value) => {
          if (!value && mapper) {
            form.reset({
              name: mapper.name,
              mapper_type: mapper.mapper_type,
              config: mapper.config ? JSON.stringify(mapper.config, null, 2) : '',
            })
          }
          onOpenChange(value)
        }}
        mapper={mapper}
        form={form}
        isValid={isValid}
        isPending={isPending}
        handleSubmit={handleSubmit}
      />
    </Form>
  )
}
