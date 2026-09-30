import { useTranslation } from 'react-i18next'
import { toast } from 'sonner'
import { Button } from '@/components/ui/button'
import { Schemas } from '@/api/api.client'
import ProviderIcon, {
  isProviderIconKey,
} from '@/components/provider-icon'
import { AUTH_NAMESPACE } from '../constants'
import { buildProviderLoginUrl } from './login-providers-helpers'

import IdentityProviderPresentation = Schemas.IdentityProviderPresentation

export type LoginProvidersProps = {
  providers: IdentityProviderPresentation[]
}

/**
 * "Or continue with" block of the login page: one button per identity
 * provider enabled in the realm. A click resolves the broker URL through
 * `buildProviderLoginUrl` and navigates to it; a failure (typically no
 * `crypto.subtle` outside HTTPS) surfaces as a toast instead of a dead click.
 */
export function LoginProviders({ providers }: LoginProvidersProps) {
  const { t } = useTranslation(AUTH_NAMESPACE)

  if (providers.length === 0) return null

  return (
    <>
      <div className='relative text-center text-xs after:absolute after:inset-0 after:top-1/2 after:z-0 after:flex after:items-center after:border-t after:border-border'>
        <span className='relative z-10 bg-card px-2 text-muted-foreground'>
          {t('providers.separator')}
        </span>
      </div>
      <div className='grid gap-2'>
        {providers.map((provider) => {
          const iconKey = provider.icon?.toLowerCase()
          const iconSrc = iconKey && !isProviderIconKey(iconKey) ? provider.icon : undefined
          return (
            <Button
              key={provider.id}
              type='button'
              variant='outline'
              className='h-10 w-full justify-start gap-2.5 border-input bg-card px-3 text-sm font-medium text-foreground shadow-none hover:bg-muted/40'
              onClick={() => {
                void buildProviderLoginUrl({
                  loginUrl: provider.login_url,
                  apiUrl: window.apiUrl,
                  currentSearch: window.location.search,
                  origin: window.location.origin,
                  pathname: window.location.pathname,
                })
                  .then((loginUrl) => {
                    window.location.href = loginUrl
                  })
                  .catch((error: unknown) => {
                    console.error(error)
                    const name = provider.display_name
                    // Only a non-secure context explains a missing crypto.subtle; storage can fail on HTTPS too.
                    toast.error(
                      window.isSecureContext
                        ? t('providers.start_failed', { name })
                        : t('providers.start_failed_insecure', { name })
                    )
                  })
              }}
            >
              <span className='flex h-5 w-5 items-center justify-center'>
                {iconKey && isProviderIconKey(iconKey) ? (
                  <ProviderIcon icon={iconKey} size='sm' className='h-4 w-4' />
                ) : iconSrc ? (
                  <img src={iconSrc} alt={provider.display_name} className='h-4 w-4' />
                ) : (
                  <span className='text-xs font-semibold uppercase text-muted-foreground'>
                    {provider.display_name?.[0] ?? provider.kind?.[0] ?? '?'}
                  </span>
                )}
              </span>
              <span className='flex-1 truncate text-left'>
                {t('providers.continue_with', { name: provider.display_name })}
              </span>
              <span className='text-xs text-muted-foreground'>→</span>
            </Button>
          )
        })}
      </div>
    </>
  )
}
