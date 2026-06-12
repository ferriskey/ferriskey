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
import {
  DeviceVerifySchema,
  USER_CODE_CHARSET,
} from '../schemas/device-verify.schema'
import './page-login.css'

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

export interface PageDeviceVerifyProps {
  status: DeviceVerifyStatus
  errorMessage: string | null
  pendingAction: 'approve' | 'deny' | null
  onSubmit: (values: DeviceVerifySchema, action: 'approve' | 'deny') => void
  onBackToStart: () => void
}

export default function PageDeviceVerify({
  status,
  errorMessage,
  pendingAction,
  onSubmit,
  onBackToStart,
}: PageDeviceVerifyProps) {
  const form = useFormContext<DeviceVerifySchema>()

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
        onSubmit={form.handleSubmit((values) => onSubmit(values, 'approve'))}
        noValidate
      >
        <div className='p-6 md:p-10'>
          <div className='flex flex-col gap-7'>
            <div className='space-y-2'>
              <div className='flex items-center gap-3'>
                <img
                  src='/logo_ferriskey.png'
                  alt='FerrisKey'
                  className='h-7 w-7 object-contain'
                />
                <p className='text-xs font-semibold uppercase tracking-[0.35em] text-muted-foreground'>
                  FerrisKey
                </p>
              </div>
              <h1 className='login-title text-3xl font-semibold tracking-tight text-foreground'>
                Device verification
              </h1>
              <p className='text-sm text-muted-foreground'>
                Enter the code displayed on your device to authorise it.
              </p>
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
                Format <span className='font-mono'>XXXX-XXXX</span>, letters only.
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

            <div className='flex flex-col gap-2'>
              <Button
                type='submit'
                className='w-full rounded-lg py-5 text-sm'
                disabled={!form.formState.isValid || isSubmitting}
              >
                {isSubmitting && pendingAction === 'approve' ? (
                  <div className='flex items-center gap-2'>
                    <BasicSpinner />
                    <span>Approving…</span>
                  </div>
                ) : (
                  'Approve'
                )}
              </Button>
              <Button
                type='button'
                variant='outline'
                className='w-full rounded-lg py-5 text-sm'
                disabled={!form.formState.isValid || isSubmitting}
                onClick={form.handleSubmit((values) => onSubmit(values, 'deny'))}
              >
                {isSubmitting && pendingAction === 'deny' ? (
                  <div className='flex items-center gap-2'>
                    <BasicSpinner />
                    <span>Denying…</span>
                  </div>
                ) : (
                  'Deny'
                )}
              </Button>
            </div>
          </div>
        </div>
      </form>
    </DeviceVerifyShell>
  )
}

function DeviceVerifyShell({ children }: { children: React.ReactNode }) {
  return (
    <div className='login-shell relative flex min-h-svh items-center justify-center px-6 py-10'>
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

function DeviceVerifyResult({
  status,
  onBackToStart,
}: {
  status: 'approved' | 'denied'
  onBackToStart: () => void
}) {
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
            {approved ? 'Device approved' : 'Access denied'}
          </h1>
          <p className='text-sm text-muted-foreground'>
            {approved
              ? 'You can now return to the device that requested access. It should sign in automatically within a few seconds.'
              : 'We told the device its request was denied. You can safely close this window.'}
          </p>
        </div>
        <Button
          type='button'
          variant='outline'
          className='w-full rounded-lg py-5 text-sm'
          onClick={onBackToStart}
        >
          Verify another code
        </Button>
      </div>
    </div>
  )
}
