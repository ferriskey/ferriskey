import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import { Form, FormField } from '@/components/ui/form'
import { UseFormReturn } from 'react-hook-form'
import { AuthenticateSchema } from '@/pages/authentication/feature/page-login-feature'
import { MagicLinkSchema } from '@/pages/authentication/schemas/magic-link.schema'
import { cn } from '@/lib/utils'
import { InputText } from '@/components/ui/input-text'
import { Link, useParams } from 'react-router'
import { Schemas } from '@/api/api.client'
import RealmLoginSetting = Schemas.RealmLoginSetting
import { LoginProviders } from './login-providers'
import './page-login.css'
import LoaderSpinner from '@/components/ui/loader-spinner'
import { Separator } from '@/components/ui/separator'
import { ArrowLeft, KeyRound, Mail, ShieldAlert } from 'lucide-react'

export type MagicLinkStep = 'idle' | 'form' | 'sent'

export interface PageLoginProps {
  form: UseFormReturn<AuthenticateSchema>
  onSubmit: (data: AuthenticateSchema) => void
  isError?: boolean
  isLoading?: boolean
  loginSettings?: RealmLoginSetting
  errorMessage?: string | null
  onPasskeyLogin?: () => void
  isPasskeyLoading?: boolean
  onMagicLinkLogin?: () => void
  isMagicLinkLoading?: boolean
  magicLinkStep?: MagicLinkStep
  magicLinkForm?: UseFormReturn<MagicLinkSchema>
  onMagicLinkSubmit?: (data: MagicLinkSchema) => void
  onMagicLinkBack?: () => void
}

export default function PageLogin({
  form,
  onSubmit,
  isError,
  isLoading,
  loginSettings,
  errorMessage,
  onPasskeyLogin,
  isPasskeyLoading,
  onMagicLinkLogin,
  isMagicLinkLoading,
  magicLinkStep,
  magicLinkForm,
  onMagicLinkSubmit,
  onMagicLinkBack,
}: PageLoginProps) {
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
              {magicLinkStep === 'form' && magicLinkForm && onMagicLinkSubmit ? (
                <MagicLinkFormView
                  form={magicLinkForm}
                  onSubmit={onMagicLinkSubmit}
                  onBack={onMagicLinkBack}
                  isLoading={isMagicLinkLoading}
                />
              ) : magicLinkStep === 'sent' ? (
                <MagicLinkSentView onBack={onMagicLinkBack} />
              ) : (
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
                        {errorMessage && (
                          <div className='rounded-md border border-destructive/30 bg-destructive/10 px-4 py-3 text-sm text-destructive'>
                            {errorMessage}
                          </div>
                        )}
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
                                autoComplete={loginSettings?.passkey_enabled ? 'username webauthn' : 'username'}
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
                          {loginSettings?.forgot_password_enabled && (
                            <div className='flex items-center'>
                              <Link
                                to={'../forgot-password'}
                                className='ml-auto text-xs font-medium text-muted-foreground underline-offset-4 transition hover:text-foreground hover:underline'
                              >
                                Forgot your password?
                              </Link>
                            </div>
                          )}
                        </div>
                        <Button type='submit' className='w-full rounded-lg py-5 text-sm'>
                          Login
                        </Button>
                        {(onPasskeyLogin || onMagicLinkLogin) && (
                          <>
                            <div className='relative'>
                              <Separator />
                              <span className='absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 bg-card px-2 text-xs text-muted-foreground'>
                                or
                              </span>
                            </div>
                            <div className='flex flex-col gap-2'>
                              {onPasskeyLogin && (
                                <Button
                                  type='button'
                                  variant='outline'
                                  className='w-full rounded-lg py-5 text-sm'
                                  onClick={onPasskeyLogin}
                                  disabled={isPasskeyLoading}
                                >
                                  <KeyRound className='mr-2 h-4 w-4' />
                                  Sign in with a passkey
                                </Button>
                              )}
                              {onMagicLinkLogin && (
                                <Button
                                  type='button'
                                  variant='outline'
                                  className='w-full rounded-lg py-5 text-sm'
                                  onClick={onMagicLinkLogin}
                                  disabled={isMagicLinkLoading}
                                >
                                  <Mail className='mr-2 h-4 w-4' />
                                  Sign in with a magic link
                                </Button>
                              )}
                            </div>
                          </>
                        )}
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
              )}
            </CardContent>
          </Card>
        </div>
      </div>
    </div>
  )
}

function MagicLinkFormView({
  form,
  onSubmit,
  onBack,
  isLoading,
}: {
  form: UseFormReturn<MagicLinkSchema>
  onSubmit: (data: MagicLinkSchema) => void
  onBack?: () => void
  isLoading?: boolean
}) {
  return (
    <Form {...form}>
      <form onSubmit={form.handleSubmit(onSubmit)}>
        <div className='p-8 md:p-10'>
          <div className='flex flex-col gap-7'>
            <div className='space-y-2'>
              {onBack && (
                <button
                  type='button'
                  onClick={onBack}
                  className='mb-1 flex items-center gap-1 text-xs text-muted-foreground transition hover:text-foreground'
                >
                  <ArrowLeft className='h-3 w-3' />
                  Back to login
                </button>
              )}
              <div className='flex items-center gap-3'>
                <img src='/logo_ferriskey.png' alt='FerrisKey' className='h-7 w-7 object-contain' />
                <p className='text-xs font-semibold uppercase tracking-[0.35em] text-muted-foreground'>
                  FerrisKey
                </p>
              </div>
              <h1 className='login-title text-3xl font-semibold tracking-tight text-foreground'>
                Sign in by email
              </h1>
              <p className='text-sm text-muted-foreground'>
                Enter your email address and we&apos;ll send you a link to sign in instantly.
              </p>
            </div>
            <FormField
              control={form.control}
              name='email'
              render={({ field }) => (
                <InputText
                  {...field}
                  label='Email address'
                  name='email'
                  type='email'
                  className='w-full'
                  autoComplete='email'
                  error={form.formState.errors.email?.message}
                />
              )}
            />
            <Button
              type='submit'
              className='w-full rounded-lg py-5 text-sm'
              disabled={isLoading}
            >
              {isLoading ? 'Sending...' : 'Send magic link'}
            </Button>
          </div>
        </div>
      </form>
    </Form>
  )
}

function MagicLinkSentView({ onBack }: { onBack?: () => void }) {
  return (
    <div className='p-8 md:p-10'>
      <div className='flex flex-col items-center gap-6 text-center'>
        <div className='flex items-center gap-3 self-start'>
          <img src='/logo_ferriskey.png' alt='FerrisKey' className='h-7 w-7 object-contain' />
          <p className='text-xs font-semibold uppercase tracking-[0.35em] text-muted-foreground'>
            FerrisKey
          </p>
        </div>
        <div className='flex h-14 w-14 items-center justify-center rounded-full bg-primary/10'>
          <Mail className='h-7 w-7 text-primary' />
        </div>
        <div className='space-y-2'>
          <h1 className='text-2xl font-semibold tracking-tight text-foreground'>Check your inbox</h1>
          <p className='text-sm text-muted-foreground'>
            We sent a magic link to your email address. Click the link to sign in — no password needed.
          </p>
          <p className='text-xs text-muted-foreground'>
            The link expires in 15 minutes. Check your spam folder if you don&apos;t see it.
          </p>
        </div>
        {onBack && (
          <Button variant='outline' onClick={onBack} className='w-full rounded-lg py-5 text-sm'>
            <ArrowLeft className='mr-2 h-4 w-4' />
            Back to login
          </Button>
        )}
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

export function LoginErrorPage({ errorMessage }: { errorMessage: string }) {
  return (
    <div className='login-shell relative flex min-h-svh items-center justify-center px-6 py-10'>
      <div className='relative z-10 w-full max-w-sm md:max-w-md lg:max-w-lg'>
        <div className='flex flex-col gap-6'>
          <Card className='login-card overflow-hidden border p-0 shadow-sm'>
            <CardContent className='grid gap-0 p-0'>
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
                      Authentication error
                    </h1>
                  </div>

                  <div className='flex flex-col items-center gap-4 py-2'>
                    <div className='flex h-12 w-12 items-center justify-center rounded-full bg-destructive/10'>
                      <ShieldAlert className='h-6 w-6 text-destructive' />
                    </div>
                    <div className='space-y-1 text-center'>
                      <p className='text-sm font-medium text-foreground'>{errorMessage}</p>
                      <p className='text-xs text-muted-foreground'>
                        Contact your administrator if this problem persists.
                      </p>
                    </div>
                  </div>
                </div>
              </div>
            </CardContent>
          </Card>
        </div>
      </div>
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
