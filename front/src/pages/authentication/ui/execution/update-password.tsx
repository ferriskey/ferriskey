import { Card, CardContent } from '@/components/ui/card.tsx'
import { InputText } from '@/components/ui/input-text.tsx'
import { Button } from '@/components/ui/button.tsx'
import { FormField } from '@/components/ui/form'
import { useFormContext } from 'react-hook-form'
import { useTranslation } from 'react-i18next'
import { Check, X } from 'lucide-react'
import { cn } from '@/lib/utils'
import type { PasswordRequirement } from '@/lib/password-policy'
import { UpdatePasswordSchema } from '../../schemas/update-password.schema'
import '../page-login.css'
import { AUTH_NAMESPACE, BRAND_NAME } from '../../constants'
import { AuthLanguageSwitcher } from '../../components/auth-language-switcher'

export interface UpdatePasswordProps {
  handleClick: () => void
  requirements: PasswordRequirement[]
}

export default function UpdatePassword({ handleClick, requirements }: UpdatePasswordProps) {
  const form = useFormContext<UpdatePasswordSchema>()
  const { t } = useTranslation(AUTH_NAMESPACE)
  const isPending = form.formState.isSubmitting
  const password = form.watch('password') ?? ''

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
                      {t('update_password.title')}
                    </h1>
                    <p className='text-sm text-muted-foreground'>
                      {t('update_password.description')}
                    </p>
                  </div>

                  <form onSubmit={handleClick}>
                    <div className='flex flex-col gap-7'>
                      <div className='grid gap-3'>
                        <FormField
                          control={form.control}
                          name='password'
                          render={({ field }) => (
                            <InputText
                              {...field}
                              label={t('update_password.password_label')}
                              name='password'
                              type='password'
                              className='w-full'
                              error={form.formState.errors.password?.message}
                            />
                          )}
                        />
                        {requirements.length > 0 && (
                          <ul
                            className='flex flex-col gap-1.5'
                            aria-label={t('update_password.requirements_label')}
                          >
                            {requirements.map((requirement) => {
                              const isMet = requirement.isMet(password)

                              return (
                                <li
                                  key={requirement.id}
                                  className={cn(
                                    'flex items-center gap-2 text-xs transition-colors',
                                    isMet ? 'text-emerald-600' : 'text-muted-foreground'
                                  )}
                                >
                                  {isMet ? (
                                    <Check className='h-3.5 w-3.5 shrink-0' aria-hidden />
                                  ) : (
                                    <X className='h-3.5 w-3.5 shrink-0' aria-hidden />
                                  )}
                                  <span>{requirement.label}</span>
                                  <span className='sr-only'>
                                    {isMet
                                      ? t('update_password.requirement_met')
                                      : t('update_password.requirement_unmet')}
                                  </span>
                                </li>
                              )
                            })}
                          </ul>
                        )}
                      </div>
                      <div className='grid gap-3'>
                        <FormField
                          control={form.control}
                          name='confirmPassword'
                          render={({ field }) => (
                            <InputText
                              {...field}
                              label={t('update_password.confirm_label')}
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
                        disabled={isPending}
                      >
                        {isPending
                          ? t('update_password.submitting')
                          : t('update_password.submit')}
                      </Button>
                    </div>
                  </form>
                </div>
              </div>
            </CardContent>
          </Card>
        </div>
      </div>
    </div>
  )
}
