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
  emailVerification: z.boolean(),
  passkey: z.boolean(),
  magicLink: z.boolean(),
  magicLinkTtl: z.number().min(1),
  loginAliases: z.array(z.enum(['username', 'email'])).min(1, { message: 'Select at least one login identifier' }),
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
      emailVerification: false,
      passkey: false,
      magicLink: false,
      magicLinkTtl: 5,
      loginAliases: ['username'] as ('username' | 'email')[],
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
        email_verification_enabled: values.emailVerification,
        passkey_enabled: values.passkey,
        magic_link_enabled: values.magicLink,
        magic_link_ttl: values.magicLinkTtl,
        login_aliases: values.loginAliases,
      }
    })
  }

  const hasChanges = useFormChanges(
    form,
    data && {
      forgotPassword: data.forgot_password_enabled,
      rememberMe: data.remember_me_enabled,
      userRegistration: data.user_registration_enabled,
      emailVerification: data.email_verification_enabled,
      passkey: data.passkey_enabled,
      magicLink: data.magic_link_enabled,
      magicLinkTtl: data.magic_link_ttl,
      loginAliases: (data.login_aliases as ('username' | 'email')[]) ?? ['username'],
    }
  )

  useEffect(() => {
    if (data) {
      form.reset({
        userRegistration: data.user_registration_enabled,
        forgotPassword: data.forgot_password_enabled,
        rememberMe: data.remember_me_enabled,
        emailVerification: data.email_verification_enabled,
        passkey: data.passkey_enabled,
        magicLink: data.magic_link_enabled,
        magicLinkTtl: data.magic_link_ttl,
        loginAliases: (data.login_aliases as ('username' | 'email')[]) ?? ['username'],
      })
    }
  }, [data, form])


  return (
    <PageRealmSettingsLogin form={form} hasChanges={hasChanges} handleSubmit={handleSubmit} />
  )
}
