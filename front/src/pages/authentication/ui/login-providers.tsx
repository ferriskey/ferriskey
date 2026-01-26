import { Button } from '@/components/ui/button'
import { Schemas } from '@/api/api.client'
import ProviderIcon, {
  isProviderIconKey,
} from '@/pages/identity-providers/components/provider-icon'

import IdentityProviderPresentation = Schemas.IdentityProviderPresentation

export type LoginProvidersProps = {
  providers: IdentityProviderPresentation[]
}

const buildProviderLoginUrl = (provider: IdentityProviderPresentation) => {
  const url = new URL(provider.login_url, window.apiUrl)
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
        <span className='relative z-10 bg-background px-2 text-muted-foreground'>
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
              className='w-full justify-start gap-3'
              onClick={() => {
                window.location.href = loginUrl
              }}
            >
              {iconKey && isProviderIconKey(iconKey) ? (
                <ProviderIcon icon={iconKey} size='sm' className='h-5 w-5' />
              ) : iconSrc ? (
                <img src={iconSrc} alt={provider.display_name} className='h-5 w-5' />
              ) : (
                <span className='flex h-5 w-5 items-center justify-center rounded-full bg-muted text-xs font-semibold uppercase text-muted-foreground'>
                  {provider.display_name?.[0] ?? provider.kind?.[0] ?? '?'}
                </span>
              )}
              <span>Continue with {provider.display_name}</span>
            </Button>
          )
        })}
      </div>
    </>
  )
}
