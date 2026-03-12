import { zodResolver } from '@hookform/resolvers/zod'
import { useForm } from 'react-hook-form'
import { z } from 'zod'
import PageRealmSettingsTokens from '../ui/page-realm-settings-tokens'
import { useGetRealm, useUpdateRealmSettings } from '@/api/realm.api'
import { useEffect } from 'react'
import { useFormChanges } from '@/hooks/use-form-changes'
import { useParams } from 'react-router'
import { RouterParams } from '@/routes/router'

const realmTokenSettingsSchema = z.object({
  accessTokenLifetime: z.number().min(60).max(86400),
  refreshTokenLifetime: z.number().min(300).max(2592000),
  idTokenLifetime: z.number().min(60).max(86400),
  temporaryTokenLifetime: z.number().min(60).max(86400),
})

export type RealmTokenSettingsSchema = z.infer<typeof realmTokenSettingsSchema>

export default function PageRealmSettingsTokensFeature() {
  const { realm_name } = useParams<RouterParams>()
  const { data: realmResponse } = useGetRealm({ realm: realm_name })
  const { mutate } = useUpdateRealmSettings()

  const form = useForm<RealmTokenSettingsSchema>({
    resolver: zodResolver(realmTokenSettingsSchema),
    defaultValues: {
      accessTokenLifetime: 300,
      refreshTokenLifetime: 86400,
      idTokenLifetime: 300,
      temporaryTokenLifetime: 300,
    }
  })

  const settings = realmResponse?.settings

  const handleSubmit = (values: RealmTokenSettingsSchema) => {
    if (!realm_name) return

    mutate({
      path: {
        name: realm_name
      },
      body: {
        access_token_lifetime: values.accessTokenLifetime,
        refresh_token_lifetime: values.refreshTokenLifetime,
        id_token_lifetime: values.idTokenLifetime,
        temporary_token_lifetime: values.temporaryTokenLifetime,
      }
    })
  }

  const hasChanges = useFormChanges(
    form,
    settings && {
      accessTokenLifetime: settings.access_token_lifetime,
      refreshTokenLifetime: settings.refresh_token_lifetime,
      idTokenLifetime: settings.id_token_lifetime,
      temporaryTokenLifetime: settings.temporary_token_lifetime,
    }
  )

  useEffect(() => {
    if (settings) {
      form.reset({
        accessTokenLifetime: settings.access_token_lifetime,
        refreshTokenLifetime: settings.refresh_token_lifetime,
        idTokenLifetime: settings.id_token_lifetime,
        temporaryTokenLifetime: settings.temporary_token_lifetime,
      })
    }
  }, [settings, form])

  return (
    <PageRealmSettingsTokens form={form} hasChanges={hasChanges} handleSubmit={handleSubmit} />
  )
}
