import { zodResolver } from '@hookform/resolvers/zod'
import { useForm } from 'react-hook-form'
import { z } from 'zod'
import PageRealmSettingsLogin from '../ui/page-realm-settings-login'
import { useGetLoginSettings, useUpdateRealmSettings } from '@/api/realm.api'
import { useEffect } from 'react'
import { useFormChanges } from '@/hooks/use-form-changes'
import { useParams } from 'react-router'
import { RouterParams } from '@/routes/router'

const realmLoginSettingsSchema = z.object({
  userRegistration: z.boolean(),
  forgotPassword: z.boolean(),
  rememberMe: z.boolean(),
})

export type RealmLoginSettingsSchema = z.infer<typeof realmLoginSettingsSchema>

export default function PageRealmSettingsLoginFeature() {
  const { realm_name } = useParams<RouterParams>()
  const { data } = useGetLoginSettings({ realm: realm_name })
  const { mutate } = useUpdateRealmSettings()

  const form = useForm<RealmLoginSettingsSchema>({
    resolver: zodResolver(realmLoginSettingsSchema),
    defaultValues: {
      forgotPassword: false,
      rememberMe: false,
      userRegistration: false,
    }
  })

  const handleSubmit = (values: RealmLoginSettingsSchema) => {
    if (!realm_name) return

    mutate({
      path: {
        name: realm_name
      },
      body: {
        forgot_password_enabled: values.forgotPassword,
        remember_me_enabled: values.rememberMe,
        user_registration_enabled: values.userRegistration,
      }
    })
  }

  const hasChanges = useFormChanges(
    form,
    data && {
      forgotPassword: data.forgot_password_enabled,
      rememberMe: data.remember_me_enabled,
      userRegistration: data.user_registration_enabled,
    }
  )

  useEffect(() => {
    if (data) {
      form.reset({
        userRegistration: data.user_registration_enabled,
        forgotPassword: data.forgot_password_enabled,
        rememberMe: data.remember_me_enabled,
      })
    }
  }, [data, form])


  return (
    <PageRealmSettingsLogin form={form} hasChanges={hasChanges} handleSubmit={handleSubmit} />
  )
}
