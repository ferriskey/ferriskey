import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import { FormField } from '@/components/ui/form'
import { InputText } from '@/components/ui/input-text'
import { UseFormReturn } from 'react-hook-form'
import { useTranslation } from 'react-i18next'
import { Link, useParams } from 'react-router'
import { type ResetPasswordSchema } from '../schemas/reset-password.schema'
import { AlertTriangle, Loader2 } from 'lucide-react'
import './page-login.css'
import type { PublicPasswordPolicy } from '@/api/password-policy.api'
import PasswordRequirements from '../components/password-requirements'
import { AUTH_NAMESPACE, BRAND_NAME } from '../constants'
import { AuthLanguageSwitcher } from '../components/auth-language-switcher'

export interface PageResetPasswordProps {
  form: UseFormReturn<ResetPasswordSchema>
  onSubmit: (data: ResetPasswordSchema) => void
  isPending: boolean
  tokenStatus: 'loading' | 'valid' | 'invalid'
  errorMessage: string | null
  policy: PublicPasswordPolicy
}

export default function PageResetPassword({
  form,
  onSubmit,
  isPending,
  tokenStatus,
  errorMessage,
  policy,
}: PageResetPasswordProps) {
  const { realm_name } = useParams()
  const { t } = useTranslation(AUTH_NAMESPACE)
  const password = form.watch('password')
  const isFormSubmittable = form.formState.isValid && !form.formState.isSubmitting && !isPending

  return (
    <div className='login-shell relative flex min-h-svh items-center justify-center px-6 py-10'>
      <AuthLanguageSwitcher />
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
                        alt={BRAND_NAME}
                        className='h-7 w-7 object-contain'
                      />
                      <p className='text-xs font-semibold uppercase tracking-[0.35em] text-muted-foreground'>
                        {BRAND_NAME}
                      </p>
                    </div>
                    <h1 className='login-title text-3xl font-semibold tracking-tight text-foreground'>
                      {t('reset_password.title')}
                    </h1>
                  </div>

                  {tokenStatus === 'loading' ? (
                    <LoadingMessage />
                  ) : tokenStatus === 'invalid' ? (
                    <InvalidTokenMessage realmName={realm_name} />
                  ) : (
                    <>
                      {errorMessage && (
                        <div className='rounded-md border border-destructive/30 bg-destructive/10 px-4 py-3 text-sm text-destructive'>
                          {errorMessage}
                        </div>
                      )}
                      <form onSubmit={form.handleSubmit(onSubmit)}>
                        <div className='flex flex-col gap-7'>
                          <div className='grid gap-3'>
                            <FormField
                              control={form.control}
                              name='password'
                              render={({ field }) => (
                                <InputText
                                  {...field}
                                  label={t('reset_password.password_label')}
                                  name='password'
                                  type='password'
                                  className='w-full'
                                  error={
                                    form.formState.errors.password?.message &&
                                    password.length > 0
                                      ? form.formState.errors.password.message
                                      : undefined
                                  }
                                />
                              )}
                            />
                            <PasswordRequirements password={password} policy={policy} />
                          </div>
                          <div className='grid gap-3'>
                            <FormField
                              control={form.control}
                              name='confirmPassword'
                              render={({ field }) => (
                                <InputText
                                  {...field}
                                  label={t('reset_password.confirm_label')}
                                  name='confirmPassword'
                                  type='password'
                                  className='w-full'
                                  error={form.formState.errors.confirmPassword?.message}
                                />
                              )}
                            />
                          </div>
                          <Button
                            type='submit'
                            className='w-full rounded-lg py-5 text-sm'
                            disabled={!isFormSubmittable}
                          >
                            {isPending
                              ? t('reset_password.submitting')
                              : t('reset_password.submit')}
                          </Button>
                        </div>
                      </form>
                    </>
                  )}
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
  const { t } = useTranslation(AUTH_NAMESPACE)

  return (
    <div className='flex flex-col items-center gap-4 py-4'>
      <Loader2 className='h-6 w-6 animate-spin text-muted-foreground' />
      <p className='text-center text-sm text-muted-foreground'>
        {t('reset_password.checking')}
      </p>
    </div>
  )
}

function InvalidTokenMessage({ realmName }: { realmName?: string }) {
  const { t } = useTranslation(AUTH_NAMESPACE)

  return (
    <div className='flex flex-col items-center gap-4 py-4'>
      <div className='flex h-12 w-12 items-center justify-center rounded-full bg-amber-100'>
        <AlertTriangle className='h-6 w-6 text-amber-600' />
      </div>
      <p className='text-center text-sm text-muted-foreground'>
        {t('reset_password.invalid')}
      </p>
      <Link
        to={`/realms/${realmName}/authentication/forgot-password`}
        className='font-semibold text-foreground underline underline-offset-4 text-sm'
      >
        {t('reset_password.request_new')}
      </Link>
    </div>
  )
}
