import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import { Form, FormField } from '@/components/ui/form'
import { UseFormReturn } from 'react-hook-form'
import { Trans, useTranslation } from 'react-i18next'
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
import { ArrowLeft, KeyRound, Mail, ShieldAlert, Wrench } from 'lucide-react'
import { AUTH_NAMESPACE, BRAND_NAME } from '../constants'

export type MagicLinkStep = 'idle' | 'form' | 'sent'

export interface PageLoginProps {
  form: UseFormReturn<AuthenticateSchema>
  onSubmit: (data: AuthenticateSchema) => void
  isError?: boolean
  isLoading?: boolean
  loginSettings?: RealmLoginSetting
  errorMessage?: string | null
  isMaintenanceError?: boolean
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
  isMaintenanceError,
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
  const { t } = useTranslation(AUTH_NAMESPACE)

  if (isError) return <ErrorMessage />
  if (isLoading) return <LoadingMessage />
  if (!loginSettings) return null

  const providers = loginSettings.identity_providers ?? []

  const aliases = loginSettings.login_aliases ?? ['username']
  const identifierLabel =
    aliases.length > 1
      ? t('login.identifier.username_or_email')
      : aliases[0] === 'email'
        ? t('login.identifier.email')
        : t('login.identifier.username')

  return (
    <div className='login-shell relative flex min-h-svh items-center justify-center px-6 py-10'>
      <div className='relative z-10 w-full max-w-[380px]'>
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
                    <div className='p-6'>
                      <div className='flex flex-col gap-5'>
                        <div className='space-y-1.5'>
                          <div className='flex items-center gap-2'>
                            <img
                              src='/logo_ferriskey.png'
                              alt={BRAND_NAME}
                              className='h-5 w-5 object-contain'
                            />
                            <p className='text-[11px] font-semibold uppercase tracking-[0.2em] text-muted-foreground'>
                              {BRAND_NAME}
                            </p>
                          </div>
                          <h1 className='login-title text-xl font-semibold tracking-tight text-foreground'>
                            {loginSettings.display_name?.trim()
                              ? loginSettings.display_name
                              : (realm_name?.toUpperCase() ?? t('login.title'))}
                          </h1>
                        </div>
                        {errorMessage && (
                          isMaintenanceError ? (
                            <div className='rounded-md border border-amber-500/30 bg-amber-500/10 px-3 py-2 text-xs text-amber-700 dark:text-amber-400 flex items-start gap-2'>
                              <Wrench className='h-4 w-4 mt-0.5 shrink-0' />
                              <span>{errorMessage}</span>
                            </div>
                          ) : (
                            <div className='rounded-md border border-destructive/30 bg-destructive/10 px-3 py-2 text-xs text-destructive'>
                              {errorMessage}
                            </div>
                          )
                        )}
                        <div className='grid gap-2'>
                          <FormField
                            control={form.control}
                            name='username'
                            render={({ field }) => (
                              <InputText
                                {...field}
                                label={identifierLabel}
                                name='username'
                                className='w-full'
                                autoComplete={loginSettings?.passkey_enabled ? 'username webauthn' : 'username'}
                                error={form.formState.errors.username?.message}
                              />
                            )}
                          />
                          <FormField
                            control={form.control}
                            name='password'
                            render={({ field }) => (
                              <InputText
                                {...field}
                                label={t('login.password')}
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
                                {t('login.forgot_password')}
                              </Link>
                            </div>
                          )}
                        </div>
                        <Button type='submit' className='h-10 w-full rounded-lg text-sm'>
                          {t('login.submit')}
                        </Button>
                        {(onPasskeyLogin || onMagicLinkLogin) && (
                          <>
                            <div className='relative'>
                              <Separator />
                              <span className='absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 bg-card px-2 text-xs text-muted-foreground'>
                                {t('login.separator')}
                              </span>
                            </div>
                            <div className='flex flex-col gap-2'>
                              {onPasskeyLogin && (
                                <Button
                                  type='button'
                                  variant='outline'
                                  className='h-10 w-full rounded-lg text-sm'
                                  onClick={onPasskeyLogin}
                                  disabled={isPasskeyLoading}
                                >
                                  <KeyRound className='mr-2 h-4 w-4' />
                                  {t('login.passkey')}
                                </Button>
                              )}
                              {onMagicLinkLogin && (
                                <Button
                                  type='button'
                                  variant='outline'
                                  className='h-10 w-full rounded-lg text-sm'
                                  onClick={onMagicLinkLogin}
                                  disabled={isMagicLinkLoading}
                                >
                                  <Mail className='mr-2 h-4 w-4' />
                                  {t('login.magic_link')}
                                </Button>
                              )}
                            </div>
                          </>
                        )}
                        <div className='space-y-3'>
                          <LoginProviders providers={providers} />
                          {loginSettings.user_registration_enabled && (
                            <div className='text-center text-xs text-muted-foreground md:text-sm'>
                              <Trans
                                i18nKey={`${AUTH_NAMESPACE}:login.no_account`}
                                components={{
                                  signup: (
                                    <Link
                                      to={'../register'}
                                      className='font-semibold text-foreground underline underline-offset-4'
                                    />
                                  ),
                                }}
                              />
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
  const { t } = useTranslation(AUTH_NAMESPACE)

  return (
    <Form {...form}>
      <form onSubmit={form.handleSubmit(onSubmit)}>
        <div className='p-6'>
          <div className='flex flex-col gap-5'>
            <div className='space-y-1.5'>
              {onBack && (
                <button
                  type='button'
                  onClick={onBack}
                  className='mb-1 flex items-center gap-1 text-xs text-muted-foreground transition hover:text-foreground'
                >
                  <ArrowLeft className='h-3 w-3' />
                  {t('actions.back_to_login')}
                </button>
              )}
              <div className='flex items-center gap-2'>
                <img src='/logo_ferriskey.png' alt={BRAND_NAME} className='h-5 w-5 object-contain' />
                <p className='text-[11px] font-semibold uppercase tracking-[0.2em] text-muted-foreground'>
                  {BRAND_NAME}
                </p>
              </div>
              <h1 className='login-title text-xl font-semibold tracking-tight text-foreground'>
                {t('magic_link.form.title')}
              </h1>
              <p className='text-sm text-muted-foreground'>
                {t('magic_link.form.description')}
              </p>
            </div>
            <FormField
              control={form.control}
              name='email'
              render={({ field }) => (
                <InputText
                  {...field}
                  label={t('magic_link.form.email_label')}
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
              className='h-10 w-full rounded-lg text-sm'
              disabled={isLoading}
            >
              {isLoading ? t('magic_link.form.submitting') : t('magic_link.form.submit')}
            </Button>
          </div>
        </div>
      </form>
    </Form>
  )
}

function MagicLinkSentView({ onBack }: { onBack?: () => void }) {
  const { t } = useTranslation(AUTH_NAMESPACE)

  return (
    <div className='p-6'>
      <div className='flex flex-col items-center gap-4 text-center'>
        <div className='flex items-center gap-3 self-start'>
          <img src='/logo_ferriskey.png' alt={BRAND_NAME} className='h-5 w-5 object-contain' />
          <p className='text-[11px] font-semibold uppercase tracking-[0.2em] text-muted-foreground'>
            {BRAND_NAME}
          </p>
        </div>
        <div className='flex h-11 w-11 items-center justify-center rounded-full bg-primary/10'>
          <Mail className='h-5 w-5 text-primary' />
        </div>
        <div className='space-y-1.5'>
          <h1 className='text-lg font-semibold tracking-tight text-foreground'>
            {t('magic_link.sent.title')}
          </h1>
          <p className='text-sm text-muted-foreground'>{t('magic_link.sent.description')}</p>
          <p className='text-xs text-muted-foreground'>{t('magic_link.sent.hint')}</p>
        </div>
        {onBack && (
          <Button variant='outline' onClick={onBack} className='h-10 w-full rounded-lg text-sm'>
            <ArrowLeft className='mr-2 h-4 w-4' />
            {t('actions.back_to_login')}
          </Button>
        )}
      </div>
    </div>
  )
}

function ErrorMessage() {
  const { t } = useTranslation(AUTH_NAMESPACE)

  return (
    <div className='flex min-h-svh flex-col items-center justify-center'>
      <p className='text-lg font-semibold text-destructive'>{t('login.failure.title')}</p>
      <p className='text-muted-foreground'>{t('login.failure.description')}</p>
    </div>
  )
}

export function LoginErrorPage({ errorMessage }: { errorMessage: string }) {
  const { t } = useTranslation(AUTH_NAMESPACE)

  return (
    <div className='login-shell relative flex min-h-svh items-center justify-center px-6 py-10'>
      <div className='relative z-10 w-full max-w-[380px]'>
        <div className='flex flex-col gap-6'>
          <Card className='login-card overflow-hidden border p-0 shadow-sm'>
            <CardContent className='grid gap-0 p-0'>
              <div className='p-6'>
                <div className='flex flex-col gap-5'>
                  <div className='space-y-1.5'>
                    <div className='flex items-center gap-2'>
                      <img
                        src='/logo_ferriskey.png'
                        alt={BRAND_NAME}
                        className='h-5 w-5 object-contain'
                      />
                      <p className='text-[11px] font-semibold uppercase tracking-[0.2em] text-muted-foreground'>
                        {BRAND_NAME}
                      </p>
                    </div>
                    <h1 className='login-title text-xl font-semibold tracking-tight text-foreground'>
                      {t('login.rejected.title')}
                    </h1>
                  </div>

                  <div className='flex flex-col items-center gap-4 py-2'>
                    <div className='flex h-10 w-10 items-center justify-center rounded-full bg-destructive/10'>
                      <ShieldAlert className='h-5 w-5 text-destructive' />
                    </div>
                    <div className='space-y-1 text-center'>
                      <p className='text-sm font-medium text-foreground'>{errorMessage}</p>
                      <p className='text-xs text-muted-foreground'>
                        {t('login.rejected.hint')}
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
