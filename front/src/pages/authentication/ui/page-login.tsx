import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import { Form, FormField } from '@/components/ui/form'
import { UseFormReturn } from 'react-hook-form'
import { AuthenticateSchema } from '@/pages/authentication/feature/page-login-feature'
import { cn } from '@/lib/utils'
import { MagicCard } from '@/components/magicui/magic-card'
import { InputText } from '@/components/ui/input-text'
import { Link } from 'react-router'
import { Schemas } from '@/api/api.client'
import appleIcon from '@/assets/icons/apple.svg'
import googleIcon from '@/assets/icons/google.svg'
import metaIcon from '@/assets/icons/meta.svg'
import RealmLoginSetting = Schemas.RealmLoginSetting
import IdentityProviderPresentation = Schemas.IdentityProviderPresentation

export interface PageLoginProps {
  form: UseFormReturn<AuthenticateSchema>
  onSubmit: (data: AuthenticateSchema) => void
  isError?: boolean
  loginSettings: RealmLoginSetting
}

export default function PageLogin({ form, onSubmit, isError, loginSettings }: PageLoginProps) {
  if (isError) return <ErrorMessage />

  const providers = loginSettings.identity_providers ?? []
  const providerIconMap: Record<string, string> = {
    apple: appleIcon,
    google: googleIcon,
    meta: metaIcon,
    facebook: metaIcon,
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

  return (
    <div className='flex min-h-svh flex-col items-center justify-center bg-muted p-6 md:p-10'>
      <div className='w-full max-w-sm md:max-w-3xl'>
        <div className={cn('flex flex-col gap-6')}>
          <Card className='overflow-hidden p-0'>
            <MagicCard className='p-0' gradientColor='#D9D9D955'>
              <CardContent className='grid p-0 md:grid-cols-2'>
                <Form {...form}>
                  <form onSubmit={form.handleSubmit(onSubmit)}>
                    <div className='p-6 md:p-8'>
                      <div className='flex flex-col gap-6'>
                        <div className='flex flex-col items-center text-center'>
                          <h1 className='text-2xl font-bold'>Welcome back</h1>
                          <p className='text-balance text-muted-foreground'>
                            Sign in to your account
                          </p>
                        </div>
                        <div className='grid gap-2'>
                          <FormField
                            control={form.control}
                            name='username'
                            render={({ field }) => (
                              <InputText
                                {...field}
                                label='Username'
                                name='username'
                                className='w-full'
                                error={form.formState.errors.username?.message}
                              />
                            )}
                          />
                        </div>
                        <div className='grid gap-2'>
                          <FormField
                            control={form.control}
                            name='password'
                            render={({ field }) => (
                              <InputText
                                {...field}
                                label='Password'
                                name='password'
                                type='password'
                                className='w-full'
                                error={form.formState.errors.password?.message}
                              />
                            )}
                          />
                          <div className='flex items-center hidden'>
                            <a
                              href='#'
                              className='ml-auto text-sm underline-offset-2 hover:underline'
                            >
                              Forgot your password?
                            </a>
                          </div>
                        </div>
                        <Button type='submit' className='w-full'>
                          Login
                        </Button>
                        {providers.length > 0 && (
                          <>
                            <div className='relative text-center text-sm after:absolute after:inset-0 after:top-1/2 after:z-0 after:flex after:items-center after:border-t after:border-border'>
                              <span className='relative z-10 bg-background px-2 text-muted-foreground'>
                                Or continue with
                              </span>
                            </div>
                            <div className='grid gap-3'>
                              {providers.map((provider) => {
                                const iconKey = provider.icon?.toLowerCase()
                                const iconSrc = iconKey ? providerIconMap[iconKey] ?? provider.icon : undefined
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
                                    {iconSrc ? (
                                      <img
                                        src={iconSrc}
                                        alt={provider.display_name}
                                        className='h-5 w-5'
                                      />
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
                        )}
                        {loginSettings.user_registration_enabled && (
                          <div className='text-center text-sm'>
                            Don&apos;t have an account?{' '}
                            <Link to={'../register'} className='underline underline-offset-4'>
                              Sign up
                            </Link>
                          </div>
                        )}
                      </div>
                    </div>
                  </form>
                </Form>
                <div className='relative hidden bg-muted md:block'>
                  <img
                    src='/logo_ferriskey.png'
                    alt='Image'
                    className='absolute inset-0 h-full w-full object-cover'
                  />
                </div>
              </CardContent>
            </MagicCard>
          </Card>
        </div>
      </div>
    </div>
  )
}

function ErrorMessage() {
  return (
    <div className='flex min-h-svh flex-col items-center justify-center'>
      <p className='text-lg font-semibold text-destructive'>An error occurred during login</p>
      <p className='text-muted-foreground'>Please try again</p>
    </div>
  )
}
