import { Card, CardContent } from '@/components/ui/card.tsx'
import { InputText } from '@/components/ui/input-text.tsx'
import { Button } from '@/components/ui/button.tsx'
import { FormField } from '@/components/ui/form'
import { useFormContext } from 'react-hook-form'
import { UpdatePasswordSchema } from '../../schemas/update-password.schema'
import '../page-login.css'

export interface UpdatePasswordProps {
  handleClick: () => void
}

export default function UpdatePassword({ handleClick }: UpdatePasswordProps) {
  const form = useFormContext<UpdatePasswordSchema>()
  const isPending = form.formState.isSubmitting

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
                      Update your password
                    </h1>
                    <p className='text-sm text-muted-foreground'>
                      Your password is temporary and must be updated before continuing.
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
                              label='New password'
                              name='password'
                              type='password'
                              className='w-full'
                              error={form.formState.errors.password?.message}
                            />
                          )}
                        />
                      </div>
                      <div className='grid gap-3'>
                        <FormField
                          control={form.control}
                          name='confirmPassword'
                          render={({ field }) => (
                            <InputText
                              {...field}
                              label='Confirm password'
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
                        {isPending ? 'Updating...' : 'Update password'}
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
