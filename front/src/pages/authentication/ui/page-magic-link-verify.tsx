import { Button } from '@/components/ui/button'
import LoaderSpinner from '@/components/ui/loader-spinner'
import { useTranslation } from 'react-i18next'
import { Link } from 'react-router'
import { AUTH_NAMESPACE } from '../constants'
import AuthShell from '../components/auth-shell'

export interface PageMagicLinkVerifyProps {
  status: 'loading' | 'error'
  errorMessage?: string | null
}

export default function PageMagicLinkVerify({ status, errorMessage }: PageMagicLinkVerifyProps) {
  const { t } = useTranslation(AUTH_NAMESPACE)

  return (
    <AuthShell title={t('magic_link.form.title')}>
      <div className='flex flex-col items-center gap-6 text-center'>
        {status === 'loading' && (
          <>
            <LoaderSpinner />
            <p className='text-sm text-muted-foreground'>{t('magic_link.verify.checking')}</p>
          </>
        )}

        {status === 'error' && (
          <>
            <div className='rounded-md border border-destructive/30 bg-destructive/10 px-4 py-3 text-sm text-destructive'>
              {errorMessage ?? t('magic_link.verify.invalid')}
            </div>
            <Button asChild variant='outline' className='w-full'>
              <Link to={'../login'}>{t('actions.back_to_login')}</Link>
            </Button>
          </>
        )}
      </div>
    </AuthShell>
  )
}
