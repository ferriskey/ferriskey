import { CheckCircle } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Link, useLocation, useParams } from 'react-router'
import { Button } from '@/components/ui/button'
import AuthShell from '../components/auth-shell'
import { AUTH_NAMESPACE } from '../constants'
import { hasAuthFlowClient, withAuthFlow } from '@/lib/auth-flow-query'

/**
 * Post-verification success screen. Reached via a redirect from
 * `/verify-email?token=…` once the backend has actually confirmed the
 * email. Kept as the React fallback when the realm admin hasn't built a
 * custom `email_verified` tree in the theme builder — when they have,
 * `<PortalLayoutWrapper>` renders the custom tree and ignores this
 * component.
 *
 * The page offers a sign-in only when it knows the client the user is signing in to. The link in
 * a verification email carries no client and usually opens in a new tab, so most of the time it
 * does not; a sign-in started from here would then fall back to the console's own client and take
 * an application's user to the console (or to `invalid_client` when the realm's console client is
 * disabled). Without a client the page tells the user to go back to their application instead.
 */
export default function PageEmailVerifiedFeature() {
  const { realm_name } = useParams()
  const { search } = useLocation()
  const { t } = useTranslation(AUTH_NAMESPACE)
  const knowsClient = hasAuthFlowClient(search)

  return (
    <AuthShell
      title={t('email_verified.title')}
      description={
        knowsClient ? t('email_verified.description') : t('email_verified.return_to_app')
      }
    >
      <div className='flex flex-col items-center gap-6 text-center'>
        <div className='flex h-14 w-14 items-center justify-center rounded-full bg-green-100'>
          <CheckCircle className='h-8 w-8 text-green-600' />
        </div>
        {knowsClient && (
          <Button asChild className='w-full rounded-lg py-5 text-sm'>
            <Link to={withAuthFlow(`/realms/${realm_name}/authentication/login`, search)}>
              {t('email_verified.action')}
            </Link>
          </Button>
        )}
      </div>
    </AuthShell>
  )
}
