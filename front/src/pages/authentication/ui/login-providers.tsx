import { Button } from '@/components/ui/button'
import { Schemas } from '@/api/api.client'
import ProviderIcon, {
  isProviderIconKey,
} from '@/pages/identity-providers/components/provider-icon'

import IdentityProviderPresentation = Schemas.IdentityProviderPresentation

export type LoginProvidersProps = {
  providers: IdentityProviderPresentation[]
}

const isAbsoluteUrl = (value: string) => /^https?:\/\//i.test(value)

const buildProviderLoginUrl = (provider: IdentityProviderPresentation) => {
  const base = window.apiUrl.endsWith('/') ? window.apiUrl : `${window.apiUrl}/`
  const path = provider.login_url.replace(/^\//, '')
  const url = new URL(isAbsoluteUrl(provider.login_url) ? provider.login_url : path, base)
  const currentParams = new URLSearchParams(window.location.search)

  currentParams.forEach((value, key) => {
    if (!url.searchParams.has(key)) {
      url.searchParams.set(key, value)
    }
  })

  return url.toString()
}

export function LoginProviders({ providers }: LoginProvidersProps) {
  if (providers.length === 0) return null

  return (
    <>
      <div className='relative text-center text-sm after:absolute after:inset-0 after:top-1/2 after:z-0 after:flex after:items-center after:border-t after:border-border'>
        <span className='relative z-10 bg-card px-2 text-muted-foreground'>
          Or continue with
        </span>
      </div>
      <div className='grid gap-3'>
        {providers.map((provider) => {
          const iconKey = provider.icon?.toLowerCase()
          const iconSrc = iconKey && !isProviderIconKey(iconKey) ? provider.icon : undefined
          const loginUrl = buildProviderLoginUrl(provider)
          return (
            <Button
              key={provider.id}
              type='button'
              variant='outline'
              className='w-full justify-start gap-3 border-input bg-card px-3 py-2 text-sm font-medium text-foreground shadow-none hover:bg-muted/40'
              onClick={() => {
                window.location.href = loginUrl
              }}
            >
              <span className='flex h-8 w-8 items-center justify-center'>
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
              <span className='flex-1 truncate text-left'>Continue with {provider.display_name}</span>
              <span className='text-xs text-muted-foreground'>→</span>
            </Button>
          )
        })}
      </div>
    </>
  )
}
