import { useMemo, useState } from 'react'
import { useParams } from 'react-router'
import { useTranslation } from 'react-i18next'
import { toast } from 'sonner'
import { useGetUserCredentials, useResetUserPassword } from '@/api/user.api'
import { useDeleteUserCredential } from '@/api/credential.api'
import { usePublicPasswordPolicy } from '@/api/password-policy.api'
import { buildSetCredentialPasswordSchema } from '@/pages/iam/user/schemas'
import { RouterParams } from '@/routes/router'
import UserCredentialsTab from '../ui/user-credentials-tab'
import type { PasswordValidity } from '../ui/user-password-form'
import { apiErrorMessage, partitionFieldErrors, validationErrorsFrom } from '@/lib/api-error'

type PasswordField = 'password' | 'confirmPassword'

const FIELD_BY_API_FIELD: Record<string, PasswordField> = {
  password: 'password',
  value: 'password',
  confirmPassword: 'confirmPassword',
}

export default function UserCredentialsFeature() {
  const { realm_name, user_id } = useParams<RouterParams>()
  const { t } = useTranslation('user')
  const realm = realm_name ?? 'master'

  const { data: credentialsResponse, isLoading } = useGetUserCredentials({
    realm,
    userId: user_id,
  })
  const { mutate: deleteCredential } = useDeleteUserCredential()
  const { mutate: resetPassword } = useResetUserPassword()
  const { data: passwordPolicy } = usePublicPasswordPolicy(realm)

  const [password, setPassword] = useState('')
  const [confirmPassword, setConfirmPassword] = useState('')
  const [validity, setValidity] = useState<PasswordValidity>('temporary')
  const [serverErrors, setServerErrors] = useState<Partial<Record<PasswordField, string>>>({})

  const schema = useMemo(
    () => buildSetCredentialPasswordSchema(passwordPolicy),
    [passwordPolicy]
  )

  const parsed = schema.safeParse({
    password,
    confirmPassword,
    temporary: validity === 'temporary',
  })

  const touched = password.length > 0 || confirmPassword.length > 0

  const clientErrors = parsed.success || !touched
    ? {}
    : {
        password: parsed.error.issues.find((i) => i.path[0] === 'password')?.message,
        confirmPassword: parsed.error.issues.find((i) => i.path[0] === 'confirmPassword')
          ?.message,
      }

  const errors = { ...clientErrors, ...serverErrors }

  const resetForm = () => {
    setPassword('')
    setConfirmPassword('')
    setValidity('temporary')
    setServerErrors({})
  }

  const changePassword = (value: string) => {
    setServerErrors({})
    setPassword(value)
  }

  const handleDelete = (credentialId: string) => {
    if (!user_id) return
    deleteCredential(
      { path: { realm_name: realm, user_id, credential_id: credentialId } },
      { onSuccess: () => toast.success(t('detail.credentials.toast.deleted')) }
    )
  }

  const handleSubmitPassword = () => {
    if (!user_id || !realm_name) {
      toast.error(t('detail.credentials.toast.missing_context'))
      return
    }
    if (!parsed.success) return

    setServerErrors({})

    resetPassword(
      {
        body: {
          credential_type: 'password',
          temporary: validity === 'temporary',
          value: password,
        },
        path: { realm_name, user_id },
      },
      {
        onSuccess: () => {
          toast.success(t('detail.credentials.toast.password_set'))
          resetForm()
        },
        onError: (error) => {
          const { byField, unattached } = partitionFieldErrors(
            validationErrorsFrom(error),
            (field) => FIELD_BY_API_FIELD[field]
          )

          setServerErrors(Object.fromEntries(byField) as Partial<Record<PasswordField, string>>)

          if (byField.size === 0 || unattached.length > 0) {
            toast.error(
              unattached.join(' — ') ||
                apiErrorMessage(error, t('detail.credentials.toast.password_failed'))
            )
          }
        },
      }
    )
  }

  return (
    <UserCredentialsTab
      credentials={credentialsResponse?.data ?? []}
      isLoading={isLoading}
      onDelete={handleDelete}
      passwordForm={{
        password,
        confirmPassword,
        validity,
        errors,
        canSubmit: parsed.success,
        onPasswordChange: changePassword,
        onConfirmPasswordChange: setConfirmPassword,
        onValidityChange: setValidity,
        onSubmit: handleSubmitPassword,
        onReset: resetForm,
      }}
    />
  )
}
