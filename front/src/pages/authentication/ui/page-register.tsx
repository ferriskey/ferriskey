import { Form, FormField } from '@/components/ui/form'
import type { UseFormReturn } from 'react-hook-form'
import { useTranslation } from 'react-i18next'
import type { RegisterSchema } from '../feature/page-register-feature'
import { InputText } from '@/components/ui/input-text'
import { Button } from '@/components/ui/button'
import type { PublicPasswordPolicy } from '@/api/password-policy.api'
import PasswordRequirements from '../components/password-requirements'
import { AUTH_NAMESPACE } from '../constants'
import AuthShell from '../components/auth-shell'

export interface PageRegisterProps {
  form: UseFormReturn<RegisterSchema>
  onSubmit: (data: RegisterSchema) => void
  backToLogin?: () => void
  policy: PublicPasswordPolicy
}

export default function PageRegister({ form, onSubmit, backToLogin, policy }: PageRegisterProps) {
  const { t } = useTranslation(AUTH_NAMESPACE)
  const password = form.watch('password')
  const isPasswordValid = !form.formState.errors.password && password.length > 0
  const isFormSubmittable =
    form.formState.isValid && !form.formState.isSubmitting

  return (
    <AuthShell title={t('register.title')}>
      <Form {...form}>
        <form onSubmit={form.handleSubmit(onSubmit)}>
          <div className='flex flex-col gap-7'>
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
        </form>
      </Form>
    </AuthShell>
  )
}
