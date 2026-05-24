import { useResendVerificationEmailMutation } from '@/api/auth.api'
import { Button } from '@/components/ui/button'
import { RouterParams } from '@/routes/router'
import { Mail, RefreshCw } from 'lucide-react'
import { useEffect, useState } from 'react'
import { useParams, useSearchParams } from 'react-router'
import { toast } from 'sonner'

const VERIFY_EMAIL_CONTEXT_KEY = 'ferriskey_verify_email_context'

export interface VerifyEmailContext {
  token: string
  realm: string
  clientId: string
  timestamp: number
}

export function storeVerifyEmailContext(context: Omit<VerifyEmailContext, 'timestamp'>) {
  sessionStorage.setItem(
    VERIFY_EMAIL_CONTEXT_KEY,
    JSON.stringify({ ...context, timestamp: Date.now() })
  )
}

export function getVerifyEmailContext(): VerifyEmailContext | null {
  const stored = sessionStorage.getItem(VERIFY_EMAIL_CONTEXT_KEY)
  if (!stored) return null

  try {
    const context = JSON.parse(stored) as VerifyEmailContext
    // Context expires after 30 minutes
    if (Date.now() - context.timestamp > 30 * 60 * 1000) {
      clearVerifyEmailContext()
      return null
    }
    return context
  } catch {
    return null
  }
}

export function clearVerifyEmailContext() {
  sessionStorage.removeItem(VERIFY_EMAIL_CONTEXT_KEY)
}

export default function VerifyEmailFeature() {
  const { realm_name } = useParams<RouterParams>()
  const [searchParams] = useSearchParams()
  const token = searchParams.get('client_data')
  const [cooldown, setCooldown] = useState(0)

  const { mutate: resendEmail, isPending } = useResendVerificationEmailMutation()

  // Store auth context for use after email verification
  useEffect(() => {
    if (token && realm_name) {
      storeVerifyEmailContext({
        token,
        realm: realm_name,
        clientId: 'security-admin-console', // TODO: get from URL params if needed
      })
    }
  }, [token, realm_name])

  useEffect(() => {
    if (cooldown > 0) {
      const timer = setTimeout(() => setCooldown(cooldown - 1), 1000)
      return () => clearTimeout(timer)
    }
  }, [cooldown])

  const handleResend = () => {
    if (!token || !realm_name) {
      toast.error('Unable to resend email: missing authentication context')
      return
    }

    resendEmail(
      { realm: realm_name, token },
      {
        onSuccess: () => {
          toast.success('Verification email sent! Check your inbox.')
          setCooldown(60) // 60 second cooldown before allowing another resend
        },
        onError: (error) => {
          toast.error(error.message || 'Failed to resend verification email')
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

        <h1 className='text-2xl font-bold'>Verify your email</h1>

        <p className='text-muted-foreground'>
          A verification link has been sent to your email address. Please check your inbox and click
          the link to verify your account.
        </p>

        <p className='text-sm text-muted-foreground'>
          If you don't see the email, check your spam folder.
        </p>

        <div className='pt-4'>
          <Button
            variant='outline'
            onClick={handleResend}
            disabled={isPending || cooldown > 0 || !token}
          >
            {isPending ? (
              <>
                <RefreshCw className='w-4 h-4 mr-2 animate-spin' />
                Sending...
              </>
            ) : cooldown > 0 ? (
              `Resend in ${cooldown}s`
            ) : (
              <>
                <RefreshCw className='w-4 h-4 mr-2' />
                Resend verification email
              </>
            )}
          </Button>
        </div>
      </div>
    </div>
  )
}
