import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import LoaderSpinner from '@/components/ui/loader-spinner'
import { useTranslation } from 'react-i18next'
import { Link } from 'react-router'
import { AUTH_NAMESPACE, BRAND_NAME } from '../constants'
import { AuthLanguageSwitcher } from '../components/auth-language-switcher'

export interface PageMagicLinkVerifyProps {
  status: 'loading' | 'error'
  errorMessage?: string | null
}

export default function PageMagicLinkVerify({ status, errorMessage }: PageMagicLinkVerifyProps) {
  const { t } = useTranslation(AUTH_NAMESPACE)

  return (
    <div className='login-shell relative flex min-h-svh items-center justify-center px-6 py-10'>
      <AuthLanguageSwitcher />
      <div className='relative z-10 w-full max-w-sm md:max-w-md'>
        <Card className='overflow-hidden border p-0 shadow-sm'>
          <CardContent className='p-8 md:p-10'>
            <div className='flex flex-col items-center gap-6 text-center'>
              <div className='flex items-center gap-3'>
                <img src='/logo_ferriskey.png' alt={BRAND_NAME} className='h-7 w-7 object-contain' />
                <p className='text-xs font-semibold uppercase tracking-[0.35em] text-muted-foreground'>
                  {BRAND_NAME}
                </p>
              </div>

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
          </CardContent>
        </Card>
      </div>
    </div>
  )
}
