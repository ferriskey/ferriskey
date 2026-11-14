import { Button } from '@/components/ui/button'
import { CheckCircle, Fingerprint, KeyRound, ShieldCheck } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { BasicSpinner } from '@/components/ui/spinner'
import { AUTH_NAMESPACE } from '../../constants'
import AuthShell from '../../components/auth-shell'

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
    <AuthShell
      title={t('configure_passkey.title')}
      description={t('configure_passkey.description')}
    >
      <div className='flex flex-col gap-7'>
        {isSuccess ? (
          <div className='flex flex-col items-center gap-3 py-4 text-center'>
            <div className='flex h-12 w-12 items-center justify-center rounded-full bg-green-500/10'>
              <CheckCircle className='h-6 w-6 text-green-600 dark:text-green-400' />
            </div>
            <p className='text-base font-semibold text-foreground'>
              {t('configure_passkey.success')}
            </p>
            <p className='text-sm text-muted-foreground'>{t('configure_passkey.redirecting')}</p>
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
    </AuthShell>
  )
}
