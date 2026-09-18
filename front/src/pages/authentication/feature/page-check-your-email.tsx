import { useLocation, useParams, Link } from 'react-router'
import { Trans, useTranslation } from 'react-i18next'
import { Mail, ArrowLeft } from 'lucide-react'
import { AUTH_NAMESPACE } from '../constants'

export default function PageCheckYourEmail() {
  const { realm_name } = useParams()
  const { t } = useTranslation(AUTH_NAMESPACE)
  const location = useLocation()
  const email = (location.state as { email?: string })?.email

  return (
    <div className='flex flex-col items-center justify-center min-h-screen p-4'>
      <div className='max-w-md w-full space-y-6 text-center'>
        <div className='mx-auto w-16 h-16 bg-blue-100 rounded-full flex items-center justify-center'>
          <Mail className='w-8 h-8 text-blue-600' />
        </div>

        <h1 className='text-2xl font-bold'>{t('check_email.title')}</h1>

        <p className='text-muted-foreground'>
          {email ? (
            <Trans
              i18nKey={`${AUTH_NAMESPACE}:check_email.description_to`}
              values={{ email }}
              components={{ recipient: <strong className='block mt-1' /> }}
            />
          ) : (
            t('check_email.description')
          )}
        </p>

        <p className='text-sm text-muted-foreground'>{t('check_email.hint')}</p>

        <Link
          to={`/realms/${realm_name}/authentication/login`}
          className='inline-flex items-center gap-2 text-sm text-primary hover:underline'
        >
          <ArrowLeft className='w-4 h-4' />
          {t('actions.back_to_login')}
        </Link>
      </div>
    </div>
  )
}
