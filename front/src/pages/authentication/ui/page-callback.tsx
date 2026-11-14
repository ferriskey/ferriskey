import LoaderSpinner from '@/components/ui/loader-spinner'
import { XCircle } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { AuthShellCenter } from '../components/auth-shell'
import { AUTH_NAMESPACE } from '../constants'

export interface PageCallbackProps {
  code?: string | null
  setup: boolean
  errorMessage?: string | null
}

export default function PageCallback({ code, setup, errorMessage }: PageCallbackProps) {
  const { t } = useTranslation(AUTH_NAMESPACE)

  if (errorMessage) {
    return <ErrorState message={errorMessage} />
  }

  if (setup && !code) {
    return <ErrorState message={t('callback.missing_code')} />
  }

  return (
    <AuthShellCenter>
      <LoaderSpinner />
    </AuthShellCenter>
  )
}

function ErrorState({ message }: { message: string }) {
  const { t } = useTranslation(AUTH_NAMESPACE)

  return (
    <AuthShellCenter>
      <div className='relative z-10 w-full max-w-md rounded-md border border-destructive/30 bg-destructive/10 p-4 text-destructive'>
        <div className='flex gap-3'>
          <div className='shrink-0 pt-0.5'>
            <XCircle aria-hidden='true' className='size-5' />
          </div>
          <div className='space-y-1'>
            <h3 className='text-sm font-medium'>{t('callback.error_title')}</h3>
            <p className='text-sm'>{message}</p>
          </div>
        </div>
      </div>
    </AuthShellCenter>
  )
}
