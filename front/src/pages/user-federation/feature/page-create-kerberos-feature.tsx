import { useNavigate, useParams } from 'react-router-dom'
import { USER_FEDERATION_OVERVIEW_URL, USER_FEDERATION_URL } from '@/routes/sub-router/user-federation.router'
import { RouterParams } from '@/routes/router'
import { toast } from 'sonner'
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import { createKerberosProviderSchema, CreateKerberosProviderSchema } from '../schemas/kerberos-provider.schema'
import { Form } from '@/components/ui/form'
import KerberosFormUi from '../ui/kerberos-form-ui'
import { useCallback, useEffect } from 'react'

export default function PageCreateKerberosFeature() {
  const navigate = useNavigate()
  const { realm_name } = useParams<RouterParams>()

  const form = useForm<CreateKerberosProviderSchema>({
    resolver: zodResolver(createKerberosProviderSchema),
    mode: 'onChange',
    defaultValues: {
      type: 'Kerberos',
      name: '',
      priority: 'Secondary',
      enabled: true,
      kerberosRealm: '',
      kdcServer: '',
      adminServer: '',
      allowPasswordAuth: true,
    },
  })

  const handleTypeChange = useCallback((newType: 'LDAP' | 'Kerberos') => {
    if (newType === 'LDAP') {
      navigate(`${USER_FEDERATION_URL(realm_name)}/ldap/create`)
    }
  }, [navigate, realm_name])

  useEffect(() => {
    handleTypeChange('LDAP')
  }, [handleTypeChange])

  const handleBack = () => {
    navigate(USER_FEDERATION_OVERVIEW_URL(realm_name))
  }

  const handleSubmit = (data: CreateKerberosProviderSchema) => {
    console.log('Creating Kerberos provider:', data)
    toast.success('Kerberos Provider created successfully')
    navigate(USER_FEDERATION_OVERVIEW_URL(realm_name))
  }

  return (
    <Form {...form}>
      <form onSubmit={form.handleSubmit(handleSubmit)}>
        <KerberosFormUi
          form={form}
          handleBack={handleBack}
          handleSubmit={form.handleSubmit(handleSubmit)}
          onTypeChange={handleTypeChange}
        />
      </form>
    </Form>
  )
}
