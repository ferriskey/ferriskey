import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import { FormField } from '@/components/ui/form'
import { InputText } from '@/components/ui/input-text'
import { UseFormReturn } from 'react-hook-form'
import { useTranslation } from 'react-i18next'
import { Link, useParams } from 'react-router'
import { type ForgotPasswordSchema } from '../schemas/forgot-password.schema'
import { CheckCircle } from 'lucide-react'
import './page-login.css'
import { AUTH_NAMESPACE, BRAND_NAME } from '../constants'
import { AuthLanguageSwitcher } from '../components/auth-language-switcher'

export interface PageForgotPasswordProps {
  form: UseFormReturn<ForgotPasswordSchema>
  onSubmit: (data: ForgotPasswordSchema) => void
  submitted: boolean
  isPending: boolean
}

export default function PageForgotPassword({ form, onSubmit, submitted, isPending }: PageForgotPasswordProps) {
  const { realm_name } = useParams()
  const { t } = useTranslation(AUTH_NAMESPACE)

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
                      {t('forgot_password.title')}
                    </h1>
                  </div>

                  {submitted ? (
                    <div className='flex flex-col items-center gap-4 py-4'>
                      <div className='flex h-12 w-12 items-center justify-center rounded-full bg-green-100'>
                        <CheckCircle className='h-6 w-6 text-green-600' />
                      </div>
                      <p className='text-center text-sm text-muted-foreground'>
                        {t('forgot_password.sent')}
                      </p>
                    </div>
                  ) : (
                    <form onSubmit={form.handleSubmit(onSubmit)}>
                      <div className='flex flex-col gap-7'>
                        <p className='text-sm text-muted-foreground'>
                          {t('forgot_password.description')}
                        </p>
                        <div className='grid gap-3'>
                          <FormField
                            control={form.control}
                            name='email'
                            render={({ field }) => (
                              <InputText
                                {...field}
                                label={t('forgot_password.email_label')}
                                name='email'
                                type='email'
                                className='w-full'
                                error={form.formState.errors.email?.message}
                              />
                            )}
                          />
                        </div>
                        <Button type='submit' className='w-full rounded-lg py-5 text-sm' disabled={isPending}>
                          {isPending
                            ? t('forgot_password.submitting')
                            : t('forgot_password.submit')}
                        </Button>
                      </div>
                    </form>
                  )}

                  <div className='text-center text-xs text-muted-foreground md:text-sm'>
                    <Link
                      to={`/realms/${realm_name}/authentication/login`}
                      className='font-semibold text-foreground underline underline-offset-4'
                    >
                      {t('actions.back_to_login')}
                    </Link>
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
