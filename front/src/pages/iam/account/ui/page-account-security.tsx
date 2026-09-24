import { useTranslation } from 'react-i18next'
import { KeyRound, ShieldCheck, Smartphone, Trash2, UserRound } from 'lucide-react'
import { QRCodeSVG } from 'qrcode.react'
import { DetailHeader, IconTile, PageShell, PageTabs, Pill, Section } from '@/components/kit'
import { Button } from '@/components/kit'
import { EmptyState } from '@/components/kit'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { ConfirmDeleteAlert } from '@/components/confirm-delete-alert'
import { useConfirmDeleteAlert } from '@/hooks/use-confirm-delete-alert.ts'
import PasswordRequirements from '@/pages/authentication/components/password-requirements'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'
import { formatDate } from '@/utils/format-date'
import { Schemas } from '@/api/api.client'
import type { PublicPasswordPolicy } from '@/api/password-policy.api'
import { useAccountTabs } from './use-account-tabs'
import ReauthenticateDialog, { ReauthenticateDialogProps } from './reauthenticate-dialog'

import User = Schemas.User

export interface PasskeySummary {
  id: string
  label: string | null
  createdAt: string
}

export interface OtpEnrollment {
  secret: string
  otpauthUri: string
}

export interface PageAccountSecurityProps {
  profile?: User
  isLoading: boolean
  policy?: PublicPasswordPolicy
  hasPassword: boolean
  hasOtp: boolean
  passkeys: PasskeySummary[]
  passkeysSupported: boolean

  currentPassword: string
  newPassword: string
  confirmPassword: string
  passwordError?: string
  isChangingPassword: boolean
  onCurrentPasswordChange: (value: string) => void
  onNewPasswordChange: (value: string) => void
  onConfirmPasswordChange: (value: string) => void
  onChangePassword: () => void

  enrollment?: OtpEnrollment
  otpCode: string
  isEnrollingOtp: boolean
  onOtpCodeChange: (value: string) => void
  onStartOtpEnrollment: () => void
  onConfirmOtpEnrollment: () => void
  onCancelOtpEnrollment: () => void
  onDisableOtp: () => void

  isAddingPasskey: boolean
  onAddPasskey: () => void
  onDeletePasskey: (credentialId: string) => void

  reauth: Omit<ReauthenticateDialogProps, 'canUseOtp'>
}

export default function PageAccountSecurity({
  profile,
  isLoading,
  policy,
  hasPassword,
  hasOtp,
  passkeys,
  passkeysSupported,
  currentPassword,
  newPassword,
  confirmPassword,
  passwordError,
  isChangingPassword,
  onCurrentPasswordChange,
  onNewPasswordChange,
  onConfirmPasswordChange,
  onChangePassword,
  enrollment,
  otpCode,
  isEnrollingOtp,
  onOtpCodeChange,
  onStartOtpEnrollment,
  onConfirmOtpEnrollment,
  onCancelOtpEnrollment,
  onDisableOtp,
  isAddingPasskey,
  onAddPasskey,
  onDeletePasskey,
  reauth,
}: PageAccountSecurityProps) {
  const { t } = useTranslation('account')
  const { tabs, tab } = useAccountTabs()
  const { confirm, ask, close } = useConfirmDeleteAlert()

  if (isLoading) {
    return (
      <PageShell>
        <div className='flex items-center gap-3'>
          <div className='size-15 animate-pulse rounded-md bg-neutral-100 dark:bg-fk-raised' />
          <div className='space-y-2'>
            <div className='h-5 w-48 animate-pulse rounded bg-neutral-100 dark:bg-fk-raised' />
            <div className='h-4 w-32 animate-pulse rounded bg-neutral-100 dark:bg-fk-raised' />
          </div>
        </div>
      </PageShell>
    )
  }

  if (!profile) {
    return (
      <PageShell>
        <div className={cn(tokens.surface.panel, 'grid place-items-center px-6 py-16')}>
          <p className='text-sm font-medium text-neutral-700 dark:text-neutral-300'>
            {t('unavailable.title')}
          </p>
          <p className='mt-1 max-w-sm text-center text-sm text-neutral-500 dark:text-neutral-400'>
            {t('unavailable.description')}
          </p>
        </div>
      </PageShell>
    )
  }

  const askDeletePasskey = (passkey: PasskeySummary) =>
    ask({
      title: t('security.passkeys.confirm.title'),
      description: t('security.passkeys.confirm.description', {
        name: passkey.label ?? t('security.passkeys.unnamed'),
      }),
      onConfirm: () => {
        onDeletePasskey(passkey.id)
        close()
      },
    })

  return (
    <PageShell>
      <DetailHeader
        icon={
          <IconTile tone='primary' className='size-15'>
            <UserRound className='size-6' strokeWidth={1.75} />
          </IconTile>
        }
        title={profile.username}
        pills={
          <>
            <Pill tone={hasOtp ? 'info' : 'neutral'} mono>
              {hasOtp ? t('security.pills.otp_on') : t('security.pills.otp_off')}
            </Pill>
            <Pill tone={passkeys.length > 0 ? 'info' : 'neutral'} mono>
              {t('security.pills.passkeys', { count: passkeys.length })}
            </Pill>
          </>
        }
      />

      <PageTabs tabs={tabs} value={tab} className='mt-5' />

      <div className={cn('mt-5', tokens.page.blockGap)}>
        <Section title={t('security.password.title')} description={t('security.password.description')}>
          {hasPassword ? (
            <div className='grid max-w-sm gap-3 py-4'>
              <div className='grid gap-1.5'>
                <Label htmlFor='account-current-password'>
                  {t('security.password.current_label')}
                </Label>
                <Input
                  id='account-current-password'
                  type='password'
                  autoComplete='current-password'
                  value={currentPassword}
                  onChange={(event) => onCurrentPasswordChange(event.target.value)}
                />
              </div>

              <div className='grid gap-1.5'>
                <Label htmlFor='account-new-password'>{t('security.password.new_label')}</Label>
                <Input
                  id='account-new-password'
                  type='password'
                  autoComplete='new-password'
                  value={newPassword}
                  onChange={(event) => onNewPasswordChange(event.target.value)}
                />
              </div>

              <div className='grid gap-1.5'>
                <Label htmlFor='account-confirm-password'>
                  {t('security.password.confirm_label')}
                </Label>
                <Input
                  id='account-confirm-password'
                  type='password'
                  autoComplete='new-password'
                  value={confirmPassword}
                  onChange={(event) => onConfirmPasswordChange(event.target.value)}
                />
              </div>

              {policy && <PasswordRequirements policy={policy} password={newPassword} />}

              {passwordError && <p className='text-xs text-fk-danger'>{passwordError}</p>}

              <Button
                className='justify-self-start'
                onClick={onChangePassword}
                disabled={isChangingPassword}
              >
                {t('security.password.submit')}
              </Button>
            </div>
          ) : (
            <EmptyState
              icon={KeyRound}
              label={t('security.password.federated_title')}
              hint={t('security.password.federated_description')}
              compact
            />
          )}
        </Section>

        <Section
          title={t('security.otp.title')}
          description={t('security.otp.description')}
          action={
            hasOtp && !enrollment ? (
              <Button variant='destructive' size='sm' onClick={onDisableOtp}>
                {t('security.otp.disable')}
              </Button>
            ) : undefined
          }
        >
          {enrollment ? (
            <div className='grid gap-4 py-4 sm:grid-cols-[auto_1fr] sm:items-start'>
              <div className='rounded-md bg-white p-3'>
                <QRCodeSVG value={enrollment.otpauthUri} size={160} />
              </div>

              <div className='grid max-w-sm gap-3'>
                <p className='text-sm text-neutral-600 dark:text-neutral-400'>
                  {t('security.otp.scan_hint')}
                </p>
                <p className='font-mono-ui text-xs break-all text-neutral-500 dark:text-neutral-400'>
                  {enrollment.secret}
                </p>

                <div className='grid gap-1.5'>
                  <Label htmlFor='account-otp-code'>{t('security.otp.code_label')}</Label>
                  <Input
                    id='account-otp-code'
                    inputMode='numeric'
                    autoComplete='one-time-code'
                    value={otpCode}
                    onChange={(event) => onOtpCodeChange(event.target.value)}
                  />
                </div>

                <div className='flex gap-2'>
                  <Button onClick={onConfirmOtpEnrollment} disabled={isEnrollingOtp}>
                    {t('security.otp.confirm')}
                  </Button>
                  <Button variant='ghost' onClick={onCancelOtpEnrollment}>
                    {t('security.otp.cancel')}
                  </Button>
                </div>
              </div>
            </div>
          ) : hasOtp ? (
            <div className='flex items-center gap-3 py-4'>
              <IconTile tone='primary'>
                <Smartphone className='size-4' strokeWidth={1.75} />
              </IconTile>
              <p className='text-sm text-neutral-600 dark:text-neutral-400'>
                {t('security.otp.enabled')}
              </p>
            </div>
          ) : (
            <div className='grid gap-3 py-4'>
              <p className='text-sm text-neutral-600 dark:text-neutral-400'>
                {t('security.otp.disabled')}
              </p>
              <Button
                className='justify-self-start'
                onClick={onStartOtpEnrollment}
                disabled={isEnrollingOtp}
              >
                {t('security.otp.enable')}
              </Button>
            </div>
          )}
        </Section>

        <Section
          title={t('security.passkeys.title')}
          description={t('security.passkeys.description')}
          action={
            passkeysSupported ? (
              <Button size='sm' onClick={onAddPasskey} disabled={isAddingPasskey}>
                {t('security.passkeys.add')}
              </Button>
            ) : undefined
          }
          divided
        >
          {!passkeysSupported ? (
            <EmptyState
              icon={ShieldCheck}
              label={t('security.passkeys.unsupported_title')}
              hint={t('security.passkeys.unsupported_description')}
              compact
            />
          ) : passkeys.length === 0 ? (
            <EmptyState
              icon={ShieldCheck}
              label={t('security.passkeys.empty_title')}
              hint={t('security.passkeys.empty_description')}
              compact
            />
          ) : (
            passkeys.map((passkey) => (
              <div key={passkey.id} className='flex items-center gap-3 py-3'>
                <IconTile tone='primary'>
                  <ShieldCheck className='size-4' strokeWidth={1.75} />
                </IconTile>
                <div className='grid min-w-0 flex-1'>
                  <span className='truncate text-sm font-medium text-neutral-900 dark:text-neutral-100'>
                    {passkey.label ?? t('security.passkeys.unnamed')}
                  </span>
                  <span className='text-xs text-neutral-500 dark:text-neutral-400'>
                    {t('security.passkeys.added', { date: formatDate(passkey.createdAt) })}
                  </span>
                </div>
                <Button
                  variant='ghost'
                  size='icon-sm'
                  aria-label={t('security.passkeys.remove')}
                  onClick={() => askDeletePasskey(passkey)}
                >
                  <Trash2 className='size-4' />
                </Button>
              </div>
            ))
          )}
        </Section>
      </div>

      <ReauthenticateDialog {...reauth} canUseOtp={hasOtp} />

      <ConfirmDeleteAlert
        title={confirm.title}
        description={confirm.description}
        open={confirm.open}
        onConfirm={confirm.onConfirm}
        onCancel={close}
      />
    </PageShell>
  )
}
