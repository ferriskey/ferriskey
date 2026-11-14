import { Button } from '@/components/ui/button'
import { FormField } from '@/components/ui/form'
import { InputText } from '@/components/ui/input-text'
import type { UseFormReturn } from 'react-hook-form'
import { useTranslation } from 'react-i18next'
import { Link, useLocation, useParams } from 'react-router'
import { withAuthFlow } from '@/lib/auth-flow-query'
import { type ForgotPasswordSchema } from '../schemas/forgot-password.schema'
import { CheckCircle } from 'lucide-react'
import { AUTH_NAMESPACE } from '../constants'
import AuthShell from '../components/auth-shell'

export interface PageForgotPasswordProps {
  form: UseFormReturn<ForgotPasswordSchema>
  onSubmit: (data: ForgotPasswordSchema) => void
  submitted: boolean
  isPending: boolean
}

export default function PageForgotPassword({ form, onSubmit, submitted, isPending }: PageForgotPasswordProps) {
  const { realm_name } = useParams()
  const { search } = useLocation()
  const { t } = useTranslation(AUTH_NAMESPACE)

  return (
    <AuthShell title={t('forgot_password.title')}>
      <div className='flex flex-col gap-7'>
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
            to={withAuthFlow(`/realms/${realm_name}/authentication/login`, search)}
            className='font-semibold text-foreground underline underline-offset-4'
          >
            {t('actions.back_to_login')}
          </Link>
        </div>
      </div>
    </AuthShell>
  )
}
