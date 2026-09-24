import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { ShieldCheck } from 'lucide-react'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Button, IconTile } from '@/components/kit'
import type { ElevationProof } from '../feature/use-elevation'

export interface ReauthenticateDialogProps {
  open: boolean
  isSubmitting: boolean
  error?: string
  requiresPassword: boolean
  canUseOtp: boolean
  onSubmit: (proof: ElevationProof) => void
  onCancel: () => void
}

interface ReauthenticateFormProps {
  isSubmitting: boolean
  error?: string
  otpAvailable: boolean
  onSubmit: (proof: ElevationProof) => void
  onCancel: () => void
}

function ReauthenticateForm({
  isSubmitting,
  error,
  otpAvailable,
  onSubmit,
  onCancel,
}: ReauthenticateFormProps) {
  const { t } = useTranslation('account')
  const [password, setPassword] = useState('')
  const [otpCode, setOtpCode] = useState('')
  const [useOtp, setUseOtp] = useState(false)

  const submit = () => {
    if (isSubmitting) return
    if (useOtp && otpAvailable) {
      if (otpCode.length === 0) return
      onSubmit({ otpCode })
      return
    }
    if (password.length === 0) return
    onSubmit({ password })
  }

  return (
    <form
      className='grid gap-3'
      onSubmit={(event) => {
        event.preventDefault()
        submit()
      }}
    >
      {useOtp && otpAvailable ? (
        <div className='grid gap-1.5'>
          <Label htmlFor='reauth-otp'>{t('reauth.otp_label')}</Label>
          <Input
            id='reauth-otp'
            inputMode='numeric'
            autoComplete='one-time-code'
            autoFocus
            value={otpCode}
            onChange={(event) => setOtpCode(event.target.value)}
          />
        </div>
      ) : (
        <div className='grid gap-1.5'>
          <Label htmlFor='reauth-password'>{t('reauth.password_label')}</Label>
          <Input
            id='reauth-password'
            type='password'
            autoComplete='current-password'
            autoFocus
            value={password}
            onChange={(event) => setPassword(event.target.value)}
          />
        </div>
      )}

      {error && <p className='text-xs text-fk-danger'>{error}</p>}

      {otpAvailable && (
        <button
          type='button'
          className='justify-self-start text-xs text-neutral-500 underline-offset-2 hover:underline dark:text-neutral-400'
          onClick={() => setUseOtp((previous) => !previous)}
        >
          {useOtp ? t('reauth.switch_to_password') : t('reauth.switch_to_otp')}
        </button>
      )}

      <DialogFooter className='mt-2'>
        <Button type='button' variant='ghost' onClick={onCancel} disabled={isSubmitting}>
          {t('reauth.cancel')}
        </Button>
        <Button type='submit' disabled={isSubmitting}>
          {t('reauth.submit')}
        </Button>
      </DialogFooter>
    </form>
  )
}

export default function ReauthenticateDialog({
  open,
  isSubmitting,
  error,
  requiresPassword,
  canUseOtp,
  onSubmit,
  onCancel,
}: ReauthenticateDialogProps) {
  const { t } = useTranslation('account')

  return (
    <Dialog open={open} onOpenChange={(next) => !next && onCancel()}>
      <DialogContent className='sm:max-w-md'>
        <DialogHeader>
          <div className='flex items-center gap-3'>
            <IconTile tone='primary'>
              <ShieldCheck className='size-4' strokeWidth={1.75} />
            </IconTile>
            <div className='grid gap-1'>
              <DialogTitle>{t('reauth.title')}</DialogTitle>
              <DialogDescription>
                {requiresPassword ? t('reauth.description_password') : t('reauth.description')}
              </DialogDescription>
            </div>
          </div>
        </DialogHeader>

        <ReauthenticateForm
          isSubmitting={isSubmitting}
          error={error}
          otpAvailable={canUseOtp && !requiresPassword}
          onSubmit={onSubmit}
          onCancel={onCancel}
        />
      </DialogContent>
    </Dialog>
  )
}
