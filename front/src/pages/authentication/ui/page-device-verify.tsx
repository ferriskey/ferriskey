import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import {
  InputOTP,
  InputOTPGroup,
  InputOTPSeparator,
  InputOTPSlot,
} from '@/components/ui/input-otp'
import { FormControl, FormField, FormItem, FormMessage } from '@/components/ui/form'
import { BasicSpinner } from '@/components/ui/spinner'
import { AlertCircle, CheckCircle2, MonitorSmartphone, XCircle } from 'lucide-react'
import { useFormContext } from 'react-hook-form'
import { Trans, useTranslation } from 'react-i18next'
import {
  DeviceVerifySchema,
  USER_CODE_CHARSET,
} from '../schemas/device-verify.schema'
import './page-login.css'
import { AUTH_NAMESPACE, BRAND_NAME, DEVICE_ACTION, type DeviceAction } from '../constants'
import { AuthLanguageSwitcher } from '../components/auth-language-switcher'

// Pattern accepted by InputOTP, restricting input to the RFC 8628 charset.
// The 8 slots are split visually by a `<InputOTPSeparator />` between
// positions 3 and 4 to render `XXXX-XXXX`; the dash itself is never stored
// in the field value (we re-insert it on submit).
const USER_CODE_PATTERN = `[${USER_CODE_CHARSET}]`

export type DeviceVerifyStatus =
  | 'idle'
  | 'submitting'
  | 'approved'
  | 'denied'
  | 'error'

export interface DeviceConsentPreview {
  client_id: string
  client_name: string
  scopes: string[]
}

export interface PageDeviceVerifyProps {
  preview?: DeviceConsentPreview
  status: DeviceVerifyStatus
  errorMessage: string | null
  pendingAction: DeviceAction | null
  onSubmit: (values: DeviceVerifySchema, action: DeviceAction) => void
  onBackToStart: () => void
}

export default function PageDeviceVerify({
  preview,
  status,
  errorMessage,
  pendingAction,
  onSubmit,
  onBackToStart,
}: PageDeviceVerifyProps) {
  const form = useFormContext<DeviceVerifySchema>()
  const { t } = useTranslation(AUTH_NAMESPACE)

  if (status === 'approved' || status === 'denied') {
    return (
      <DeviceVerifyShell>
        <DeviceVerifyResult status={status} onBackToStart={onBackToStart} />
      </DeviceVerifyShell>
    )
  }

  const isSubmitting = status === 'submitting'

  return (
    <DeviceVerifyShell>
      <form
        onSubmit={form.handleSubmit((values) => onSubmit(values, DEVICE_ACTION.APPROVE))}
        noValidate
      >
        <div className='p-6 md:p-10'>
          <div className='flex flex-col gap-7'>
            <div className='space-y-2'>
              <div className='flex items-center gap-3'>
                <img
                  src='/logo_ferriskey.png'
                  alt={BRAND_NAME}
                  className='h-7 w-7 object-contain'
                />
                <p className='text-xs font-semibold uppercase tracking-[0.35em] text-muted-foreground'>
                  {BRAND_NAME}
                </p>
              </div>
              <h1 className='login-title text-3xl font-semibold tracking-tight text-foreground'>
                {t('device.title')}
              </h1>
              <p className='text-sm text-muted-foreground'>{t('device.description')}</p>
            </div>

            <div className='flex flex-col items-center gap-3'>
              <div className='flex h-12 w-12 items-center justify-center rounded-full bg-primary/10'>
                <MonitorSmartphone className='h-6 w-6 text-primary' />
              </div>
              <FormField
                control={form.control}
                name='user_code'
                render={({ field }) => {
                  // Strip the dash so InputOTP sees a flat 8-char string;
                  // the schema re-adds it on submit via the regex pattern.
                  const raw = field.value?.replace('-', '') ?? ''
                  return (
                    <FormItem className='flex flex-col items-center gap-2'>
                      <FormControl>
                        <InputOTP
                          maxLength={8}
                          pattern={USER_CODE_PATTERN}
                          value={raw}
                          onChange={(next) => {
                            const upper = next.toUpperCase()
                            field.onChange(
                              upper.length > 4
                                ? `${upper.slice(0, 4)}-${upper.slice(4)}`
                                : upper
                            )
                          }}
                          autoFocus
                        >
                          <InputOTPGroup>
                            <InputOTPSlot className='h-10 w-9 sm:h-11 sm:w-11 uppercase' index={0} />
                            <InputOTPSlot className='h-10 w-9 sm:h-11 sm:w-11 uppercase' index={1} />
                            <InputOTPSlot className='h-10 w-9 sm:h-11 sm:w-11 uppercase' index={2} />
                            <InputOTPSlot className='h-10 w-9 sm:h-11 sm:w-11 uppercase' index={3} />
                          </InputOTPGroup>
                          <InputOTPSeparator />
                          <InputOTPGroup>
                            <InputOTPSlot className='h-10 w-9 sm:h-11 sm:w-11 uppercase' index={4} />
                            <InputOTPSlot className='h-10 w-9 sm:h-11 sm:w-11 uppercase' index={5} />
                            <InputOTPSlot className='h-10 w-9 sm:h-11 sm:w-11 uppercase' index={6} />
                            <InputOTPSlot className='h-10 w-9 sm:h-11 sm:w-11 uppercase' index={7} />
                          </InputOTPGroup>
                        </InputOTP>
                      </FormControl>
                      <FormMessage />
                    </FormItem>
                  )
                }}
              />
              <p className='text-xs text-muted-foreground'>
                <Trans
                  i18nKey={`${AUTH_NAMESPACE}:device.format_hint`}
                  components={{ mono: <span className='font-mono' /> }}
                />
              </p>
            </div>

            {status === 'error' && errorMessage && (
              <div
                role='alert'
                className='flex items-start gap-2 rounded-md border border-destructive/40 bg-destructive/5 p-3 text-sm text-destructive'
              >
                <AlertCircle className='mt-0.5 h-4 w-4 shrink-0' />
                <span>{errorMessage}</span>
              </div>
            )}

            {preview && (
              <div className='flex flex-col gap-2 rounded-lg border border-border bg-muted/30 p-3 text-sm'>
                <p className='text-muted-foreground'>
                  <Trans
                    i18nKey={`${AUTH_NAMESPACE}:device.consent.request`}
                    values={{ name: preview.client_name || preview.client_id }}
                    components={{ app: <span className='font-medium text-foreground' /> }}
                  />
                </p>
                {preview.scopes.length > 0 && (
                  <div className='flex flex-col gap-1'>
                    <p className='text-xs uppercase tracking-wide text-muted-foreground'>
                      {t('device.consent.grants')}
                    </p>
                    <ul className='flex flex-wrap gap-1'>
                      {preview.scopes.map((scope) => (
                        <li
                          key={scope}
                          className='rounded-md bg-primary/10 px-2 py-0.5 text-xs text-foreground'
                        >
                          {scope}
                        </li>
                      ))}
                    </ul>
                  </div>
                )}
              </div>
            )}

            <div className='flex flex-col gap-2'>
              <Button
                type='submit'
                className='w-full rounded-lg py-5 text-sm'
                disabled={!form.formState.isValid || isSubmitting}
              >
                {isSubmitting && pendingAction === DEVICE_ACTION.APPROVE ? (
                  <div className='flex items-center gap-2'>
                    <BasicSpinner />
                    <span>{t('device.approving')}</span>
                  </div>
                ) : (
                  t('device.approve')
                )}
              </Button>
              <Button
                type='button'
                variant='outline'
                className='w-full rounded-lg py-5 text-sm'
                disabled={!form.formState.isValid || isSubmitting}
                onClick={form.handleSubmit((values) => onSubmit(values, DEVICE_ACTION.DENY))}
              >
                {isSubmitting && pendingAction === DEVICE_ACTION.DENY ? (
                  <div className='flex items-center gap-2'>
                    <BasicSpinner />
                    <span>{t('device.denying')}</span>
                  </div>
                ) : (
                  t('device.deny')
                )}
              </Button>
            </div>
          </div>
        </div>
      </form>
    </DeviceVerifyShell>
  )
}

export function DeviceVerifyShell({ children }: { children: React.ReactNode }) {
  return (
    <div className='login-shell relative flex min-h-svh items-center justify-center px-6 py-10'>
      <AuthLanguageSwitcher />
      <div className='relative z-10 w-full max-w-sm md:max-w-md lg:max-w-lg'>
        <div className='flex flex-col gap-6'>
          <Card className='login-card overflow-hidden border p-0 shadow-sm'>
            <CardContent className='grid gap-0 p-0'>{children}</CardContent>
          </Card>
        </div>
      </div>
    </div>
  )
}

export function DeviceVerifyResult({
  status,
  onBackToStart,
}: {
  status: 'approved' | 'denied'
  onBackToStart: () => void
}) {
  const { t } = useTranslation(AUTH_NAMESPACE)
  const approved = status === 'approved'
  const Icon = approved ? CheckCircle2 : XCircle
  return (
    <div className='p-6 md:p-10'>
      <div className='flex flex-col items-center gap-5 text-center'>
        <div
          className={
            'flex h-14 w-14 items-center justify-center rounded-full ' +
            (approved
              ? 'bg-emerald-500/10 text-emerald-600'
              : 'bg-destructive/10 text-destructive')
          }
        >
          <Icon className='h-7 w-7' />
        </div>
        <div className='space-y-2'>
          <h1 className='login-title text-2xl font-semibold tracking-tight text-foreground'>
            {approved ? t('device.approved.title') : t('device.denied.title')}
          </h1>
          <p className='text-sm text-muted-foreground'>
            {approved ? t('device.approved.description') : t('device.denied.description')}
          </p>
        </div>
        <Button
          type='button'
          variant='outline'
          className='w-full rounded-lg py-5 text-sm'
          onClick={onBackToStart}
        >
          {t('device.retry')}
        </Button>
      </div>
    </div>
  )
}
