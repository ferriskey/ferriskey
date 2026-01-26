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
import RealmLoginSetting = Schemas.RealmLoginSetting
import { LoginProviders } from './login-providers'

export interface PageLoginProps {
  form: UseFormReturn<AuthenticateSchema>
  onSubmit: (data: AuthenticateSchema) => void
  isError?: boolean
  loginSettings: RealmLoginSetting
}

export default function PageLogin({ form, onSubmit, isError, loginSettings }: PageLoginProps) {
  if (isError) return <ErrorMessage />

  const providers = loginSettings.identity_providers ?? []

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
                        <LoginProviders providers={providers} />
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
