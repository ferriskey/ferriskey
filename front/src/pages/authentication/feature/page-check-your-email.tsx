import { useLocation, useParams, Link } from 'react-router'
import { Trans, useTranslation } from 'react-i18next'
import { Mail, ArrowLeft } from 'lucide-react'
import AuthShell from '../components/auth-shell'
import { AUTH_NAMESPACE } from '../constants'
import { withAuthFlow } from '@/lib/auth-flow-query'

export default function PageCheckYourEmail() {
  const { realm_name } = useParams()
  const { t } = useTranslation(AUTH_NAMESPACE)
  const location = useLocation()
  const email = (location.state as { email?: string })?.email

  return (
    <AuthShell
      title={t('check_email.title')}
      description={
        email ? (
          <Trans
            i18nKey={`${AUTH_NAMESPACE}:check_email.description_to`}
            values={{ email }}
            components={{ recipient: <strong className='block mt-1' /> }}
          />
        ) : (
          t('check_email.description')
        )
      }
    >
      <div className='space-y-6 text-center'>
        <div className='mx-auto w-16 h-16 bg-blue-100 rounded-full flex items-center justify-center'>
          <Mail className='w-8 h-8 text-blue-600' />
        </div>

        <p className='text-sm text-muted-foreground'>{t('check_email.hint')}</p>

        <Link
          to={withAuthFlow(`/realms/${realm_name}/authentication/login`, location.search)}
          className='inline-flex items-center gap-2 text-sm text-primary hover:underline'
        >
          <ArrowLeft className='w-4 h-4' />
          {t('actions.back_to_login')}
        </Link>
      </div>
    </AuthShell>
  )
}
