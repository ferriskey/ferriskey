import { Form } from '@/components/ui/form'
import { zodResolver } from '@hookform/resolvers/zod'
import { useEffect, useMemo } from 'react'
import { useForm } from 'react-hook-form'
import { useNavigate, useParams } from 'react-router'
import { RouterParams } from '@/routes/router'
import {
  CLIENT_SCOPES_OVERVIEW_URL,
  CLIENT_SCOPES_URL,
} from '@/routes/sub-router/client-scope.router'
import { useCreateClientScope } from '@/api/client-scope.api'
import PageCreateClientScope from '../ui/page-create-client-scope'
import {
  CreateClientScopeSchema,
  createClientScopeSchema,
} from '../schemas/create-client-scope.schema'

export default function PageCreateClientScopeFeature() {
  const { realm_name } = useParams<RouterParams>()
  const navigate = useNavigate()
  const { mutate: createClientScope, data: responseCreateClientScope } = useCreateClientScope()

  const form = useForm<CreateClientScopeSchema>({
    resolver: zodResolver(createClientScopeSchema),
    mode: 'onChange',
    defaultValues: {
      name: '',
      description: '',
      protocol: 'openid-connect',
      scopeType: 'optional',
    },
  })

  const url = useMemo(() => {
    if (!realm_name) return ''
    return `${CLIENT_SCOPES_URL(realm_name)}${CLIENT_SCOPES_OVERVIEW_URL}`
  }, [realm_name])

  const handleSubmit = form.handleSubmit((values) => {
    if (!realm_name) return

    const trimmedDescription = values.description?.trim() || null
    createClientScope({
      path: {
        realm_name,
      },
      body: {
        name: values.name,
        description: trimmedDescription,
        protocol: values.protocol,
        is_default: values.scopeType === 'default',
      },
    })
  })

  const handleBack = () => {
    if (!url) return
    navigate(url)
  }

  useEffect(() => {
    if (responseCreateClientScope) {
      navigate(url)
    }
  }, [navigate, responseCreateClientScope, url])

  const formIsValid = form.formState.isValid && form.formState.isDirty

  return (
    <Form {...form}>
      <PageCreateClientScope
        form={form}
        handleBack={handleBack}
        handleSubmit={handleSubmit}
        formIsValid={formIsValid}
      />
    </Form>
  )
}
