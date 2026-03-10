import { useDeleteClientScope, useGetClientScope, useUpdateClientScope } from '@/api/client-scope.api'
import { Form } from '@/components/ui/form'
import { zodResolver } from '@hookform/resolvers/zod'
import { useEffect } from 'react'
import { useForm } from 'react-hook-form'
import { RouterParams } from '@/routes/router'
import { useNavigate, useParams } from 'react-router'
import PageClientScopeDetail from '../ui/page-client-scope-detail'
import {
  UpdateClientScopeSchema,
  updateClientScopeSchema,
} from '../schemas/update-client-scope.schema'
import { CLIENT_SCOPES_OVERVIEW_URL, CLIENT_SCOPES_URL } from '@/routes/sub-router/client-scope.router'

export default function PageClientScopeDetailFeature() {
  const { realm_name, scope_id } = useParams<RouterParams>()
  const navigate = useNavigate()

  const { data: scope, isLoading } = useGetClientScope({
    realm: realm_name ?? 'master',
    scopeId: scope_id,
  })

  const { mutate: updateClientScope, isPending } = useUpdateClientScope()
  const { mutate: deleteClientScope } = useDeleteClientScope()

  const form = useForm<UpdateClientScopeSchema>({
    resolver: zodResolver(updateClientScopeSchema),
    mode: 'onChange',
    defaultValues: {
      name: '',
      description: '',
      scopeType: 'optional',
    },
  })

  useEffect(() => {
    if (scope) {
      form.reset({
        name: scope.name,
        description: scope.description ?? '',
        scopeType: scope.default_scope_type === 'DEFAULT' ? 'default' : 'optional',
      })
    }
  }, [scope, form])

  const handleSubmit = form.handleSubmit((values) => {
    if (!realm_name || !scope_id) return

    updateClientScope({
      path: { realm_name, scope_id },
      body: {
        name: values.name,
        description: values.description?.trim() || null,
        protocol: scope?.protocol,
        is_default: values.scopeType === 'default',
      },
    })
  })

  const handleReset = () => {
    if (scope) {
      form.reset({
        name: scope.name,
        description: scope.description ?? '',
        scopeType: scope.default_scope_type === 'DEFAULT' ? 'default' : 'optional',
      })
    }
  }

  const handleDelete = () => {
    if (!realm_name || !scope_id) return
    deleteClientScope(
      { path: { realm_name, scope_id } },
      {
        onSuccess: () => {
          navigate(`${CLIENT_SCOPES_URL(realm_name)}${CLIENT_SCOPES_OVERVIEW_URL}`)
        },
      }
    )
  }

  if (!scope) {
    return (
      <div className='text-sm text-muted-foreground'>
        {isLoading ? 'Loading client scope details...' : 'Client scope not found.'}
      </div>
    )
  }

  const formIsValid = form.formState.isValid && form.formState.isDirty && !isPending

  return (
    <Form {...form}>
      <PageClientScopeDetail
        scope={scope}
        isLoading={isLoading}
        form={form}
        formIsValid={formIsValid}
        isPending={isPending}
        handleSubmit={handleSubmit}
        handleReset={handleReset}
        handleDelete={handleDelete}
      />
    </Form>
  )
}
