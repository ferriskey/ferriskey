import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import { InputOTP, InputOTPGroup, InputOTPSlot } from '@/components/ui/input-otp'
import { REGEXP_ONLY_DIGITS } from 'input-otp'
import { ShieldCheck } from 'lucide-react'
import { ChallengeOtpSchema } from '../schemas/challange-otp.schema'
import { useFormContext } from 'react-hook-form'
import { FormControl, FormField, FormItem } from '@/components/ui/form'
import { BasicSpinner } from '@/components/ui/spinner'
import './page-login.css'

export interface PageOtpChallengeProps {
  handleCancelClick: () => void
  handleClick: (values: ChallengeOtpSchema) => void
  email?: string
  isLoading?: boolean
}

export default function PageOtpChallenge({
  handleCancelClick,
  handleClick,
  email,
  isLoading,
}: PageOtpChallengeProps) {
  const form = useFormContext<ChallengeOtpSchema>()

  return (
    <div className='login-shell relative flex min-h-svh items-center justify-center px-6 py-10'>
      <div className='relative z-10 w-full max-w-sm md:max-w-md lg:max-w-lg'>
        <div className='flex flex-col gap-6'>
          <Card className='login-card overflow-hidden border p-0 shadow-sm'>
            <CardContent className='grid gap-0 p-0'>
              <form onSubmit={form.handleSubmit(handleClick)}>
                <div className='p-6 md:p-10'>
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
                        Verification code
                      </h1>
                      <p className='text-sm text-muted-foreground'>
                        Enter the 6-digit code from your authenticator app
                        {email && (
                          <>
                            {' '}for{' '}
                            <span className='font-medium text-foreground'>{email}</span>
                          </>
                        )}
                        .
                      </p>
                    </div>

                    <div className='flex flex-col items-center gap-3'>
                      <div className='flex h-12 w-12 items-center justify-center rounded-full bg-primary/10'>
                        <ShieldCheck className='h-6 w-6 text-primary' />
                      </div>
                      <FormField
                        control={form.control}
                        name='code'
                        render={({ field }) => (
                          <FormItem>
                            <FormControl>
                              <InputOTP {...field} maxLength={6} pattern={REGEXP_ONLY_DIGITS}>
                                <div className='flex items-center gap-1 sm:gap-3'>
                                  <InputOTPGroup>
                                    <InputOTPSlot className='h-10 w-9 sm:h-11 sm:w-11' index={0} />
                                  </InputOTPGroup>
                                  <InputOTPGroup>
                                    <InputOTPSlot className='h-10 w-9 sm:h-11 sm:w-11' index={1} />
                                  </InputOTPGroup>
                                  <InputOTPGroup>
                                    <InputOTPSlot className='h-10 w-9 sm:h-11 sm:w-11' index={2} />
                                  </InputOTPGroup>
                                  <InputOTPGroup>
                                    <InputOTPSlot className='h-10 w-9 sm:h-11 sm:w-11' index={3} />
                                  </InputOTPGroup>
                                  <InputOTPGroup>
                                    <InputOTPSlot className='h-10 w-9 sm:h-11 sm:w-11' index={4} />
                                  </InputOTPGroup>
                                  <InputOTPGroup>
                                    <InputOTPSlot className='h-10 w-9 sm:h-11 sm:w-11' index={5} />
                                  </InputOTPGroup>
                                </div>
                              </InputOTP>
                            </FormControl>
                          </FormItem>
                        )}
                      />
                      <p className='text-xs text-muted-foreground'>
                        Paste your code or enter it digit by digit.
                      </p>
                    </div>

                    <div className='flex flex-col gap-2'>
                      <Button
                        type='submit'
                        className='w-full rounded-lg py-5 text-sm'
                        disabled={!form.formState.isValid || isLoading}
                      >
                        {isLoading ? (
                          <div className='flex items-center gap-2'>
                            <BasicSpinner />
                            <span>Signing in...</span>
                          </div>
                        ) : (
                          'Sign in'
                        )}
                      </Button>
                      <Button
                        type='button'
                        variant='outline'
                        className='w-full rounded-lg py-5 text-sm'
                        onClick={handleCancelClick}
                      >
                        Cancel
                      </Button>
                    </div>
                  </div>
                </div>
              </form>
            </CardContent>
          </Card>
        </div>
      </div>
    </div>
  )
}
