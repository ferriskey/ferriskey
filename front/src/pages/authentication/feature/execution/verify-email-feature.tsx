import { useResendVerificationEmailMutation } from '@/api/auth.api'
import { Button } from '@/components/ui/button'
import { RouterParams } from '@/routes/router'
import { Mail, RefreshCw } from 'lucide-react'
import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useParams } from 'react-router'
import { toast } from 'sonner'
import { apiErrorMessage } from '@/lib/api-error'
import { AUTH_NAMESPACE } from '../../constants'
import { storeVerifyEmailContext } from './verify-email-context'

const RESEND_COOLDOWN_SECONDS = 60

export default function VerifyEmailFeature() {
  const { realm_name } = useParams<RouterParams>()
  const { t } = useTranslation(AUTH_NAMESPACE)
  const [cooldown, setCooldown] = useState(0)

  const { mutate: resendEmail, isPending } = useResendVerificationEmailMutation()

  // Store auth context for use after email verification
  useEffect(() => {
    if (realm_name) {
      storeVerifyEmailContext({
        realm: realm_name,
        clientId: 'security-admin-console', // TODO: get from URL params if needed
      })
    }
  }, [realm_name])

  useEffect(() => {
    if (cooldown > 0) {
      const timer = setTimeout(() => setCooldown(cooldown - 1), 1000)
      return () => clearTimeout(timer)
    }
  }, [cooldown])

  const handleResend = () => {
    if (!realm_name) {
      toast.error(t('verify_email.missing_context'))
      return
    }

    resendEmail(
      { realm: realm_name },
      {
        onSuccess: () => {
          toast.success(t('verify_email.sent'))
          setCooldown(RESEND_COOLDOWN_SECONDS)
        },
        onError: (error) => {
          toast.error(apiErrorMessage(error, t('verify_email.resend_failed')))
        },
      }
    )
  }

  return (
    <div className='flex flex-col items-center justify-center min-h-screen p-4'>
      <div className='max-w-md w-full space-y-6 text-center'>
        <div className='mx-auto w-16 h-16 bg-blue-100 rounded-full flex items-center justify-center'>
          <Mail className='w-8 h-8 text-blue-600' />
        </div>

        <h1 className='text-2xl font-bold'>{t('verify_email.title')}</h1>

        <p className='text-muted-foreground'>{t('verify_email.description')}</p>

        <p className='text-sm text-muted-foreground'>{t('verify_email.spam_hint')}</p>

        <div className='pt-4'>
          <Button
            variant='outline'
            onClick={handleResend}
            disabled={isPending || cooldown > 0}
          >
            {isPending ? (
              <>
                <RefreshCw className='w-4 h-4 mr-2 animate-spin' />
                {t('verify_email.resending')}
              </>
            ) : cooldown > 0 ? (
              t('verify_email.resend_in', { seconds: cooldown })
            ) : (
              <>
                <RefreshCw className='w-4 h-4 mr-2' />
                {t('verify_email.resend')}
              </>
            )}
          </Button>
        </div>
      </div>
    </div>
  )
}
