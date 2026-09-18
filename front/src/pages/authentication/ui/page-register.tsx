import { Card, CardContent } from '@/components/ui/card'
import { Form, FormField } from '@/components/ui/form'
import { UseFormReturn } from 'react-hook-form'
import { useTranslation } from 'react-i18next'
import { RegisterSchema } from '../feature/page-register-feature'
import { InputText } from '@/components/ui/input-text'
import { Button } from '@/components/ui/button'
import { useParams } from 'react-router'
import './page-login.css'
import type { PublicPasswordPolicy } from '@/api/password-policy.api'
import PasswordRequirements from '../components/password-requirements'
import { AUTH_NAMESPACE, BRAND_NAME } from '../constants'
import { AuthLanguageSwitcher } from '../components/auth-language-switcher'

export interface PageRegisterProps {
  form: UseFormReturn<RegisterSchema>
  onSubmit: (data: RegisterSchema) => void
  backToLogin?: () => void
  policy: PublicPasswordPolicy
}

export default function PageRegister({ form, onSubmit, backToLogin, policy }: PageRegisterProps) {
  const { realm_name } = useParams()
  const { t } = useTranslation(AUTH_NAMESPACE)
  const password = form.watch('password')
  const isPasswordValid = !form.formState.errors.password && password.length > 0
  const isFormSubmittable =
    form.formState.isValid && !form.formState.isSubmitting

  return (
    <div className='login-shell flex min-h-svh flex-col items-center justify-center px-6 py-10'>
      <AuthLanguageSwitcher />
      <div className='w-full max-w-sm md:max-w-md lg:max-w-lg'>
        <div className='flex flex-col gap-6'>
          <Card className='login-card overflow-hidden border p-0 shadow-sm'>
            <CardContent className='p-0'>
              <Form {...form}>
                <form onSubmit={form.handleSubmit(onSubmit)}>
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
                        <div className='space-y-1'>
                          <h1 className='login-title text-3xl font-semibold tracking-tight text-foreground md:text-4xl'>
                            {realm_name ?? t('register.title')}
                          </h1>
                        </div>
                      </div>

                      <div className='grid gap-6'>
                        <div className='grid gap-2'>
                          <FormField
                            control={form.control}
                            name='username'
                            render={({ field, fieldState }) => (
                              <div className='flex flex-col gap-1'>
                                <InputText {...field} label={t('register.username')} className='w-full' />
                                {fieldState.error && (
                                  <p className='text-sm text-destructive'>
                                    {fieldState.error.message}
                                  </p>
                                )}
                              </div>
                            )}
                          />

                          <FormField
                            control={form.control}
                            name='email'
                            render={({ field, fieldState }) => (
                              <div className='flex flex-col gap-1'>
                                <InputText {...field} label={t('register.email')} className='w-full' />
                                {fieldState.error && (
                                  <p className='text-sm text-destructive'>
                                    {fieldState.error.message}
                                  </p>
                                )}
                              </div>
                            )}
                          />
                        </div>

                        <div className='grid gap-2'>
                          <FormField
                            control={form.control}
                            name='password'
                            render={({ field, fieldState }) => (
                              <div className='flex flex-col gap-1'>
                                <InputText
                                  {...field}
                                  label={t('register.password')}
                                  type='password'
                                  className='w-full'
                                />
                                {fieldState.error && isPasswordValid === false && password.length > 0 && (
                                  <p className='text-sm text-destructive'>
                                    {fieldState.error.message}
                                  </p>
                                )}
                              </div>
                            )}
                          />
                          <PasswordRequirements password={password} policy={policy} />

                          <FormField
                            control={form.control}
                            name='confirmPassword'
                            render={({ field, fieldState }) => (
                              <div className='flex flex-col gap-1'>
                                <InputText
                                  {...field}
                                  label={t('register.confirm_password')}
                                  type='password'
                                  className='w-full'
                                />
                                {fieldState.error && (
                                  <p className='text-sm text-destructive'>
                                    {fieldState.error.message}
                                  </p>
                                )}
                              </div>
                            )}
                          />
                        </div>

                        <div className='grid grid-cols-2 gap-2'>
                          <FormField
                            control={form.control}
                            name='firstName'
                            render={({ field, fieldState }) => (
                              <div className='flex flex-col gap-1'>
                                <InputText {...field} label={t('register.firstname')} className='w-full' />
                                {fieldState.error && (
                                  <p className='text-sm text-destructive'>
                                    {fieldState.error.message}
                                  </p>
                                )}
                              </div>
                            )}
                          />

                          <FormField
                            control={form.control}
                            name='lastName'
                            render={({ field, fieldState }) => (
                              <div className='flex flex-col gap-1'>
                                <InputText {...field} label={t('register.lastname')} className='w-full' />
                                {fieldState.error && (
                                  <p className='text-sm text-destructive'>
                                    {fieldState.error.message}
                                  </p>
                                )}
                              </div>
                            )}
                          />
                        </div>
                      </div>

                      <div className='flex flex-col gap-2'>
                        <Button className='w-full' disabled={!isFormSubmittable}>
                          {t('register.submit')}
                        </Button>

                        <Button type='button' variant='outline' onClick={backToLogin} className='w-full'>
                          {t('actions.back_to_login')}
                        </Button>
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
