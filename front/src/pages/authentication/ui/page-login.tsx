import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import { Form, FormField } from '@/components/ui/form'
import { UseFormReturn } from 'react-hook-form'
import { AuthenticateSchema } from '@/pages/authentication/feature/page-login-feature'
import { cn } from '@/lib/utils'
import { InputText } from '@/components/ui/input-text'
import { Link, useParams } from 'react-router'
import { Schemas } from '@/api/api.client'
import RealmLoginSetting = Schemas.RealmLoginSetting
import { LoginProviders } from './login-providers'
import './page-login.css'
import LoaderSpinner from '@/components/ui/loader-spinner'

export interface PageLoginProps {
  form: UseFormReturn<AuthenticateSchema>
  onSubmit: (data: AuthenticateSchema) => void
  isError?: boolean
  isLoading?: boolean
  loginSettings?: RealmLoginSetting
}

export default function PageLogin({ form, onSubmit, isError, isLoading, loginSettings }: PageLoginProps) {
  const { realm_name } = useParams()

  if (isError) return <ErrorMessage />
  if (isLoading) return <LoadingMessage />
  if (!loginSettings) return null

  const providers = loginSettings.identity_providers ?? []

  return (
    <div className='login-shell relative flex min-h-svh items-center justify-center px-6 py-10'>
      <div className='relative z-10 w-full max-w-sm md:max-w-md lg:max-w-lg'>
        <div className={cn('flex flex-col gap-6')}>
          <Card className='login-card overflow-hidden border p-0 shadow-sm'>
            <CardContent className='grid gap-0 p-0'>
              <Form {...form}>
                <form onSubmit={form.handleSubmit(onSubmit)}>
                  <div className='p-8 md:p-10'>
                    <div className='flex flex-col gap-7'>
                      <div className='space-y-2'>
                        <div className='flex items-center gap-3'>
                          <img
                            src='/logo_ferriskey.png'
                            alt='FerrisKey'
                            className='h-7 w-7 object-contain'
                          />
                          <p className='text-xs font-semibold uppercase tracking-[0.35em] text-muted-foreground'>
                            FerrisKey
                          </p>
                        </div>
                        <h1 className='login-title text-3xl font-semibold tracking-tight text-foreground'>
                          {realm_name?.toUpperCase() ?? 'Login'}
                        </h1>
                      </div>
                      <div className='grid gap-3'>
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
                      <div className='grid gap-3'>
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
                        <div className='hidden items-center'>
                          <a
                            href='#'
                            className='ml-auto text-xs font-medium text-muted-foreground underline-offset-4 transition hover:text-foreground hover:underline'
                          >
                            Forgot your password?
                          </a>
                        </div>
                      </div>
                      <Button type='submit' className='w-full rounded-lg py-5 text-sm'>
                        Login
                      </Button>
                      <div className='space-y-4'>
                        <LoginProviders providers={providers} />
                        {loginSettings.user_registration_enabled && (
                          <div className='text-center text-xs text-muted-foreground md:text-sm'>
                            Don&apos;t have an account?{' '}
                            <Link to={'../register'} className='font-semibold text-foreground underline underline-offset-4'>
                              Sign up
                            </Link>
                          </div>
                        )}
                      </div>
                    </div>
                  </div>
                </form>
              </Form>
            </CardContent>
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

function LoadingMessage() {
  return (
    <div className='login-shell flex min-h-svh items-center justify-center'>
      <LoaderSpinner />
    </div>
  )
}
