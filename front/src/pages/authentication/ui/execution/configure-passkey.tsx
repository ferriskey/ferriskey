import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import { CheckCircle, Fingerprint, KeyRound, ShieldCheck } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { BasicSpinner } from '@/components/ui/spinner'
import '../page-login.css'
import { AUTH_NAMESPACE, BRAND_NAME } from '../../constants'

export interface ConfigurePasskeyProps {
  onRegister: () => void
  isLoading: boolean
  isSuccess: boolean
}

export default function ConfigurePasskey({
  onRegister,
  isLoading,
  isSuccess,
}: ConfigurePasskeyProps) {
  const { t } = useTranslation(AUTH_NAMESPACE)

  return (
    <div className='login-shell relative flex min-h-svh items-center justify-center px-6 py-10'>
      <div className='relative z-10 w-full max-w-sm md:max-w-md lg:max-w-lg'>
        <div className='flex flex-col gap-6'>
          <Card className='login-card overflow-hidden border p-0 shadow-sm'>
            <CardContent className='grid gap-0 p-0'>
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
                      {t('configure_passkey.title')}
                    </h1>
                    <p className='text-sm text-muted-foreground'>
                      {t('configure_passkey.description')}
                    </p>
                  </div>

                  {isSuccess ? (
                    <div className='flex flex-col items-center gap-3 py-4 text-center'>
                      <div className='flex h-12 w-12 items-center justify-center rounded-full bg-green-500/10'>
                        <CheckCircle className='h-6 w-6 text-green-600 dark:text-green-400' />
                      </div>
                      <p className='text-base font-semibold text-foreground'>
                        {t('configure_passkey.success')}
                      </p>
                      <p className='text-sm text-muted-foreground'>
                        {t('configure_passkey.redirecting')}
                      </p>
                    </div>
                  ) : (
                    <>
                      <div className='flex flex-col items-center gap-3'>
                        <div className='flex h-12 w-12 items-center justify-center rounded-full bg-primary/10'>
                          <Fingerprint className='h-6 w-6 text-primary' />
                        </div>
                      </div>

                      <div className='rounded-lg border border-border bg-muted/30 p-4 space-y-3'>
                        <div className='flex items-start gap-3'>
                          <ShieldCheck className='h-5 w-5 text-primary mt-0.5 shrink-0' />
                          <div>
                            <p className='text-sm font-medium text-foreground'>
                              {t('configure_passkey.benefits.secure.title')}
                            </p>
                            <p className='text-xs text-muted-foreground'>
                              {t('configure_passkey.benefits.secure.description')}
                            </p>
                          </div>
                        </div>
                        <div className='flex items-start gap-3'>
                          <Fingerprint className='h-5 w-5 text-primary mt-0.5 shrink-0' />
                          <div>
                            <p className='text-sm font-medium text-foreground'>
                              {t('configure_passkey.benefits.portable.title')}
                            </p>
                            <p className='text-xs text-muted-foreground'>
                              {t('configure_passkey.benefits.portable.description')}
                            </p>
                          </div>
                        </div>
                      </div>

                      <Button
                        type='button'
                        className='w-full rounded-lg py-5 text-sm'
                        onClick={onRegister}
                        disabled={isLoading}
                      >
                        {isLoading ? (
                          <div className='flex items-center gap-2'>
                            <BasicSpinner />
                            <span>{t('configure_passkey.submitting')}</span>
                          </div>
                        ) : (
                          <>
                            <KeyRound className='mr-2 h-4 w-4' />
                            {t('configure_passkey.submit')}
                          </>
                        )}
                      </Button>
                    </>
                  )}
                </div>
              </div>
            </CardContent>
          </Card>
        </div>
      </div>
    </div>
  )
}
