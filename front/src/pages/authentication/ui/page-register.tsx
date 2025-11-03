import { MagicCard } from '@/components/magicui/magic-card'
import { Card, CardContent } from '@/components/ui/card'
import { Form, FormField } from '@/components/ui/form'
import { UseFormReturn } from 'react-hook-form'
import { RegisterSchema } from '../feature/page-register-feature'
import { InputText } from '@/components/ui/input-text'
import { Button } from '@/components/ui/button'

export interface PageRegisterProps {
  form: UseFormReturn<RegisterSchema>
  onSubmit: (data: RegisterSchema) => void
  backToLogin?: () => void
}

export default function PageRegister({ form, onSubmit, backToLogin }: PageRegisterProps) {


  return (
    <div className='flex min-h-svh flex-col items-center justify-center bg-muted p-6 md:p-10'>
      <div className='w-full max-w-sm md:max-w-xl'>
        <div className='flex flex-col gap-6'>
          <Card className='overflow-hidden p-0'>
            <MagicCard
              className='p-0'
              gradientColor='#D9D9D955'
            >
              <CardContent className='p-0'>
                <Form {...form}>
                  <form onSubmit={form.handleSubmit(onSubmit)}>
                    <div className='p-6 md:p-8'>
                      <div className='flex flex-col gap-6'>
                        <div className='flex flex-col items-center text-center'>
                          <h1 className='text-2xl font-bold'>
                            Create an account
                          </h1>
                          <p className='text-balance text-muted-foreground'>
                            Register a new account to get started.
                          </p>
                        </div>

                        <div className='grid gap-10'>
                          <div className='grid gap-2'>
                            <FormField
                              control={form.control}
                              name='username'
                              render={({ field }) => (
                                <InputText
                                  {...field}
                                  label='Username'
                                  name='username'
                                  className='w-full'
                                />
                              )}
                            />

                            <FormField
                              control={form.control}
                              name='email'
                              render={({ field }) => (
                                <InputText
                                  {...field}
                                  label='Email'
                                  name='email'
                                  className='w-full'
                                />
                              )}
                            />
                          </div>

                          <div className='grid gap-2'>
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
                                />
                              )}
                            />

                            <FormField
                              control={form.control}
                              name='confirmPassword'
                              render={({ field }) => (
                                <InputText
                                  {...field}
                                  label='Confirm Password'
                                  name='confirm_password'
                                  type='password'
                                  className='w-full'
                                />
                              )}
                            />
                          </div>


                          <div className='grid grid-cols-2 gap-2'>
                            <FormField
                              control={form.control}
                              name='firstName'
                              render={({ field }) => (
                                <InputText
                                  {...field}
                                  label='Firstname'
                                  name='firstname'
                                  className='w-full'
                                />
                              )}
                            />

                            <FormField
                              control={form.control}
                              name='lastName'
                              render={({ field }) => (
                                <InputText
                                  {...field}
                                  label='Lastname'
                                  name='lastname'
                                  className='w-full'
                                />
                              )}
                            />

                          </div>
                        </div>


                        <div className='flex flex-col gap-2'>
                          <Button disabled={!form.formState.isValid}>
                            Create Account
                          </Button>

                          <Button variant='secondary' onClick={backToLogin}>
                            Back to login
                          </Button>
                        </div>
                      </div>

                    </div>
                  </form>

                </Form>
              </CardContent>

            </MagicCard>

          </Card>
        </div>
      </div>



    </div>
  )

}
