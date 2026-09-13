import { useParams } from 'react-router'
import { toast } from 'sonner'
import { useGetRealm, useUpdateRealmSettings } from '@/api/realm.api'
import { useGetSmtpConfig } from '@/api/smtp.api'
import { RouterParams } from '@/routes/router'
import { Schemas } from '@/api/api.client'
import { useDraft } from '@/pages/iam/realm/feature/use-draft'
import PageSignInMethods, {
  type SignInDraft,
  type SignInErrors,
} from '../ui/page-sign-in-methods'

import LoginAlias = Schemas.LoginAlias

const DEFAULT_DRAFT: SignInDraft = {
  loginAliases: ['username'],
  passkey: false,
  magicLink: false,
  magicLinkTtl: 15,
  requireMfa: false,
  userRegistration: false,
  emailVerification: false,
  forgotPassword: false,
  rememberMe: false,
  lockoutThreshold: 0,
  lockoutDuration: 0,
}

export default function PageSignInMethodsFeature() {
  const { realm_name } = useParams<RouterParams>()
  const realm = realm_name ?? 'master'

  const { data: realmData, isLoading } = useGetRealm({ realm })
  const { data: smtpConfig, isError: smtpFailed } = useGetSmtpConfig({ realm })
  const { mutate: updateSettings, isPending } = useUpdateRealmSettings()

  const settings = realmData?.settings

  const pristine: SignInDraft = settings
    ? {
        loginAliases:
          settings.login_aliases.length > 0
            ? settings.login_aliases
            : (['username'] as LoginAlias[]),
        passkey: settings.passkey_enabled,
        magicLink: settings.magic_link_enabled,
        magicLinkTtl: settings.magic_link_ttl,
        requireMfa: settings.require_mfa,
        userRegistration: settings.user_registration_enabled,
        emailVerification: settings.email_verification_enabled,
        forgotPassword: settings.forgot_password_enabled,
        rememberMe: settings.remember_me_enabled,
        lockoutThreshold: settings.lockout_threshold,
        lockoutDuration: settings.lockout_duration_seconds,
      }
    : DEFAULT_DRAFT

  const draft = useDraft(settings ? `${settings.id}:${settings.updated_at}` : '', pristine)
  const value = draft.value

  const errors: SignInErrors = {}
  if (value.loginAliases.length === 0) {
    errors.loginAliases = 'Select at least one identifier'
  }
  if (value.magicLink && (!Number.isFinite(value.magicLinkTtl) || value.magicLinkTtl < 1)) {
    errors.magicLinkTtl = 'A magic link must stay valid for at least one minute'
  }
  if (!Number.isFinite(value.lockoutThreshold) || value.lockoutThreshold < 0) {
    errors.lockoutThreshold = 'Cannot be negative'
  }
  if (!Number.isFinite(value.lockoutDuration) || value.lockoutDuration < 0) {
    errors.lockoutDuration = 'Cannot be negative'
  }
  if (value.lockoutThreshold > 0 && value.lockoutDuration === 0) {
    errors.lockoutDuration = 'A lockout with no duration never releases the account'
  }

  const changed = (Object.keys(pristine) as (keyof SignInDraft)[]).filter((key) =>
    key === 'loginAliases'
      ? value.loginAliases.join() !== pristine.loginAliases.join()
      : value[key] !== pristine[key]
  )

  const canSave = Object.keys(errors).length === 0

  const save = () => {
    if (!canSave || changed.length === 0 || isPending) return

    updateSettings(
      {
        path: { name: realm },
        body: {
          login_aliases: value.loginAliases,
          passkey_enabled: value.passkey,
          magic_link_enabled: value.magicLink,
          magic_link_ttl: value.magicLinkTtl,
          require_mfa: value.requireMfa,
          user_registration_enabled: value.userRegistration,
          email_verification_enabled: value.emailVerification,
          forgot_password_enabled: value.forgotPassword,
          remember_me_enabled: value.rememberMe,
          lockout_threshold: value.lockoutThreshold,
          lockout_duration_seconds: value.lockoutDuration,
        },
      },
      {
        onSuccess: () => toast.success('Sign-in methods updated.'),
        onError: (error: Error) =>
          toast.error(error.message || 'Failed to update the sign-in methods'),
      }
    )
  }

  return (
    <PageSignInMethods
      value={value}
      errors={errors}
      isLoading={isLoading}
      notFound={!isLoading && !settings}
      smtpConfigured={Boolean(smtpConfig) && !smtpFailed}
      dirtyCount={changed.length}
      canSave={canSave}
      isSaving={isPending}
      onChange={draft.patch}
      onDiscard={draft.reset}
      onSave={save}
    />
  )
}
