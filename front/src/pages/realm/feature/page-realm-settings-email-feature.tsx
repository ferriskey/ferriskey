import { zodResolver } from '@hookform/resolvers/zod'
import { useForm } from 'react-hook-form'
import { z } from 'zod'
import { useEffect } from 'react'
import { useParams } from 'react-router'
import { RouterParams } from '@/routes/router'
import { useFormChanges } from '@/hooks/use-form-changes'
import { useGetSmtpConfig, useUpsertSmtpConfig, useDeleteSmtpConfig } from '@/api/smtp.api'
import PageRealmSettingsEmail from '../ui/page-realm-settings-email'

const smtpConfigSchema = z.object({
  host: z.string().min(1, 'Host is required'),
  port: z.coerce.number().min(1).max(65535),
  username: z.string().min(1, 'Username is required'),
  password: z.string(),
  from_email: z.string().email('Must be a valid email'),
  from_name: z.string().min(1, 'From name is required'),
  encryption: z.enum(['tls', 'starttls', 'none']),
})

export type SmtpConfigSchema = z.infer<typeof smtpConfigSchema>

export default function PageRealmSettingsEmailFeature() {
  const { realm_name } = useParams<RouterParams>()
  const { data, isError } = useGetSmtpConfig({ realm: realm_name })
  const { mutate: upsert } = useUpsertSmtpConfig()
  const { mutate: remove } = useDeleteSmtpConfig()

  const hasConfig = !!data && !isError

  const form = useForm<SmtpConfigSchema>({
    resolver: zodResolver(smtpConfigSchema),
    defaultValues: {
      host: '',
      port: 587,
      username: '',
      password: '',
      from_email: '',
      from_name: '',
      encryption: 'tls',
    },
  })

  useEffect(() => {
    if (data && !isError) {
      form.reset({
        host: data.host,
        port: data.port,
        username: data.username,
        password: '',
        from_email: data.from_email,
        from_name: data.from_name,
        encryption: (data.encryption as 'tls' | 'starttls' | 'none') ?? 'tls',
      })
    }
  }, [data, isError, form])

  const defaultValues: SmtpConfigSchema = {
    host: '',
    port: 587,
    username: '',
    password: '',
    from_email: '',
    from_name: '',
    encryption: 'tls',
  }

  const hasChanges = useFormChanges(
    form,
    data && !isError
      ? {
          host: data.host,
          port: data.port,
          username: data.username,
          password: '',
          from_email: data.from_email,
          from_name: data.from_name,
          encryption: (data.encryption as 'tls' | 'starttls' | 'none') ?? 'tls',
        }
      : defaultValues
  )

  const handleSubmit = (values: SmtpConfigSchema) => {
    if (!realm_name) return

    upsert({
      path: { realm_name },
      body: values,
    })
  }

  const handleDelete = () => {
    if (!realm_name) return

    remove(
      { path: { realm_name } },
      {
        onSuccess: () => {
          form.reset({
            host: '',
            port: 587,
            username: '',
            password: '',
            from_email: '',
            from_name: '',
            encryption: 'tls',
          })
        },
      }
    )
  }

  return (
    <PageRealmSettingsEmail
      form={form}
      hasChanges={hasChanges}
      hasConfig={hasConfig}
      handleSubmit={handleSubmit}
      handleDelete={handleDelete}
    />
  )
}
