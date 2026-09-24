import { useState } from 'react'
import { useParams } from 'react-router'
import { useTranslation } from 'react-i18next'
import { toast } from 'sonner'
import {
  useChangeOwnPassword,
  useConfirmOwnOtpEnrollment,
  useConfirmOwnPasskeyRegistration,
  useDeleteOwnPasskey,
  useDisableOwnOtp,
  useGetOwnCredentials,
  useStartOwnOtpEnrollment,
  useStartOwnPasskeyRegistration,
} from '@/api/account-security.api'
import { useGetOwnProfile } from '@/api/user.api'
import { usePublicPasswordPolicy } from '@/api/password-policy.api'
import { apiErrorMessage } from '@/lib/api-error'
import { isWebAuthnAvailable, startRegistration } from '@/lib/webauthn'
import { RouterParams } from '@/routes/router'
import PageAccountSecurity, { OtpEnrollment, PasskeySummary } from '../ui/page-account-security'
import { useElevation } from './use-elevation'

const PASSWORD = 'password'
const OTP = 'otp'
const PASSKEY = 'webauthn-public-key-credential'

export default function PageAccountSecurityFeature() {
  const { realm_name } = useParams<RouterParams>()
  const { t } = useTranslation('account')
  const realm = realm_name ?? 'master'

  const { data: profileResponse, isLoading } = useGetOwnProfile({ realm })
  const { data: credentialsResponse } = useGetOwnCredentials({ realm })
  const { data: policy } = usePublicPasswordPolicy(realm)

  const elevation = useElevation(realm)

  const { mutateAsync: changePassword, isPending: isChangingPassword } = useChangeOwnPassword()
  const { mutateAsync: startOtp } = useStartOwnOtpEnrollment()
  const { mutateAsync: confirmOtp, isPending: isConfirmingOtp } = useConfirmOwnOtpEnrollment()
  const { mutateAsync: disableOtp } = useDisableOwnOtp()
  const { mutateAsync: startPasskey } = useStartOwnPasskeyRegistration()
  const { mutateAsync: confirmPasskey, isPending: isConfirmingPasskey } =
    useConfirmOwnPasskeyRegistration()
  const { mutateAsync: deletePasskey } = useDeleteOwnPasskey()

  const [currentPassword, setCurrentPassword] = useState('')
  const [newPassword, setNewPassword] = useState('')
  const [confirmPassword, setConfirmPassword] = useState('')
  const [passwordError, setPasswordError] = useState<string | undefined>()

  const [enrollment, setEnrollment] = useState<OtpEnrollment | undefined>()
  const [otpCode, setOtpCode] = useState('')
  const [isStartingOtp, setIsStartingOtp] = useState(false)
  const [isAddingPasskey, setIsAddingPasskey] = useState(false)

  const profile = profileResponse?.data
  const credentials = credentialsResponse?.data ?? []

  const hasPassword = credentials.some((credential) => credential.credential_type === PASSWORD)
  const hasOtp = credentials.some((credential) => credential.credential_type === OTP)
  const passkeys: PasskeySummary[] = credentials
    .filter((credential) => credential.credential_type === PASSKEY)
    .map((credential) => ({
      id: credential.id,
      label: credential.label ?? null,
      createdAt: credential.created_at,
    }))

  const path = { realm_name: realm }

  const fail = (caught: unknown) => toast.error(apiErrorMessage(caught))

  const onChangePassword = () => {
    setPasswordError(undefined)

    if (newPassword !== confirmPassword) {
      setPasswordError(t('security.password.mismatch'))
      return
    }
    if (currentPassword.length === 0 || newPassword.length === 0) {
      setPasswordError(t('security.password.required'))
      return
    }

    void elevation
      .run(async (elevationId) => {
        await changePassword({
          path,
          body: {
            elevation_id: elevationId,
            current_password: currentPassword,
            new_password: newPassword,
          },
        })

        setCurrentPassword('')
        setNewPassword('')
        setConfirmPassword('')
        toast.success(t('security.password.toast'))
      })
      .catch(fail)
  }

  const onStartOtpEnrollment = () => {
    setIsStartingOtp(true)

    void elevation
      .run(
        async (elevationId) => {
          const started = await startOtp({ path, body: { elevation_id: elevationId } })
          setEnrollment({ secret: started.secret, otpauthUri: started.otpauth_uri })
          setOtpCode('')
        },
        { requiresPassword: true }
      )
      .catch(fail)
      .finally(() => setIsStartingOtp(false))
  }

  const onConfirmOtpEnrollment = () => {
    void elevation
      .run(
        async (elevationId) => {
          await confirmOtp({ path, body: { elevation_id: elevationId, code: otpCode } })
          setEnrollment(undefined)
          setOtpCode('')
          toast.success(t('security.otp.toast_enabled'))
        },
        { requiresPassword: true }
      )
      .catch(fail)
  }

  const onDisableOtp = () => {
    void elevation
      .run(
        async (elevationId) => {
          await disableOtp({ path, body: { elevation_id: elevationId } })
          toast.success(t('security.otp.toast_disabled'))
        },
        { requiresPassword: true }
      )
      .catch(fail)
  }

  const onAddPasskey = () => {
    setIsAddingPasskey(true)

    void elevation
      .run(
        async (elevationId) => {
          const options = await startPasskey({ path, body: { elevation_id: elevationId } })
          const credential = await startRegistration(
            options.publicKey as Record<string, unknown>
          )

          await confirmPasskey({
            path,
            body: { elevation_id: elevationId, credential },
          })

          toast.success(t('security.passkeys.toast_added'))
        },
        { requiresPassword: true }
      )
      .catch((caught) => {
        if (caught instanceof DOMException && caught.name === 'NotAllowedError') return
        fail(caught)
      })
      .finally(() => setIsAddingPasskey(false))
  }

  const onDeletePasskey = (credentialId: string) => {
    void elevation
      .run(
        async (elevationId) => {
          await deletePasskey({
            path: { realm_name: realm, credential_id: credentialId },
            body: { elevation_id: elevationId },
          })
          toast.success(t('security.passkeys.toast_removed'))
        },
        { requiresPassword: true }
      )
      .catch(fail)
  }

  return (
    <PageAccountSecurity
      profile={profile}
      isLoading={isLoading}
      policy={policy}
      hasPassword={hasPassword}
      hasOtp={hasOtp}
      passkeys={passkeys}
      passkeysSupported={isWebAuthnAvailable()}
      currentPassword={currentPassword}
      newPassword={newPassword}
      confirmPassword={confirmPassword}
      passwordError={passwordError}
      isChangingPassword={isChangingPassword}
      onCurrentPasswordChange={setCurrentPassword}
      onNewPasswordChange={setNewPassword}
      onConfirmPasswordChange={setConfirmPassword}
      onChangePassword={onChangePassword}
      enrollment={enrollment}
      otpCode={otpCode}
      isEnrollingOtp={isStartingOtp || isConfirmingOtp}
      onOtpCodeChange={setOtpCode}
      onStartOtpEnrollment={onStartOtpEnrollment}
      onConfirmOtpEnrollment={onConfirmOtpEnrollment}
      onCancelOtpEnrollment={() => {
        setEnrollment(undefined)
        setOtpCode('')
      }}
      onDisableOtp={onDisableOtp}
      isAddingPasskey={isAddingPasskey || isConfirmingPasskey}
      onAddPasskey={onAddPasskey}
      onDeletePasskey={onDeletePasskey}
      reauth={{
        open: elevation.isOpen,
        isSubmitting: elevation.isSubmitting,
        error: elevation.error,
        requiresPassword: elevation.requiresPassword,
        onSubmit: (proof) => void elevation.submit(proof),
        onCancel: elevation.cancel,
      }}
    />
  )
}
