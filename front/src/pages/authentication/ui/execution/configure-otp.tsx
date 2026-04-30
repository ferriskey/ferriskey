import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import { InputText } from '@/components/ui/input-text'
import { Separator } from '@/components/ui/separator'
import { CheckCircle, Copy, ShieldCheck } from 'lucide-react'
import { useState } from 'react'
import { QRCodeSVG } from 'qrcode.react'
import { InputOTP, InputOTPGroup, InputOTPSlot } from '@/components/ui/input-otp'
import { REGEXP_ONLY_DIGITS } from 'input-otp'
import { Skeleton } from '@/components/ui/skeleton'
import { VerifyOtpSchema } from '../../schemas/verify-otp.schema'
import { useFormContext } from 'react-hook-form'
import { FormControl, FormField, FormItem } from '@/components/ui/form'
import '../page-login.css'

export interface ConfigureOtpProps {
  secret?: string
  qrCodeUrl?: string
  handleSubmit: (values: VerifyOtpSchema) => void
  handleCancel?: () => void
}

export default function ConfigureOtp({
  secret,
  qrCodeUrl,
  handleSubmit,
  handleCancel,
}: ConfigureOtpProps) {
  const [secretCopied, setSecretCopied] = useState<boolean>(false)
  const form = useFormContext<VerifyOtpSchema>()

  const copySecret = () => {
    if (!secret) return
    navigator.clipboard.writeText(secret)
    setSecretCopied(true)
    setTimeout(() => setSecretCopied(false), 2000)
  }

  return (
    <div className='login-shell relative flex min-h-svh items-center justify-center px-6 py-10'>
      <div className='relative z-10 w-full max-w-sm md:max-w-md lg:max-w-lg'>
        <div className='flex flex-col gap-6'>
          <Card className='login-card overflow-hidden border p-0 shadow-sm'>
            <CardContent className='grid gap-0 p-0'>
              <form onSubmit={form.handleSubmit(handleSubmit)}>
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
                        Enable two-factor authentication
                      </h1>
                      <p className='text-sm text-muted-foreground'>
                        Secure your account with an additional layer of protection using a TOTP
                        authenticator app.
                      </p>
                    </div>

                    <Step number={1} title='Install an authenticator app'>
                      <p className='text-sm text-muted-foreground'>
                        Use Google Authenticator, Authy, Microsoft Authenticator, 1Password or any
                        other TOTP app on your device.
                      </p>
                    </Step>

                    <Separator />

                    <Step number={2} title='Scan the QR code'>
                      <div className='flex justify-center rounded-lg border bg-background p-4'>
                        {qrCodeUrl ? (
                          <QRCodeSVG
                            value={qrCodeUrl}
                            size={160}
                            bgColor='transparent'
                            fgColor='currentColor'
                          />
                        ) : (
                          <Skeleton className='h-40 w-40' />
                        )}
                      </div>
                      <div className='space-y-2'>
                        <p className='text-xs text-muted-foreground'>
                          Can't scan? Enter this secret manually:
                        </p>
                        <div className='flex items-center gap-2'>
                          {secret ? (
                            <code className='flex-1 break-all rounded-md bg-muted p-2 font-mono text-xs'>
                              {secret}
                            </code>
                          ) : (
                            <Skeleton className='h-8 w-full' />
                          )}
                          <Button
                            type='button'
                            variant='outline'
                            size='sm'
                            onClick={copySecret}
                            className='shrink-0'
                          >
                            {secretCopied ? (
                              <CheckCircle className='h-4 w-4 text-green-500' />
                            ) : (
                              <Copy className='h-4 w-4' />
                            )}
                          </Button>
                        </div>
                      </div>
                    </Step>

                    <Separator />

                    <Step number={3} title='Verify and name your device'>
                      <div className='flex flex-col items-center gap-3'>
                        <FormField
                          control={form.control}
                          name='pin'
                          render={({ field }) => (
                            <FormItem>
                              <FormControl>
                                <InputOTP
                                  {...field}
                                  maxLength={6}
                                  pattern={REGEXP_ONLY_DIGITS}
                                >
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
                      </div>
                      <FormField
                        control={form.control}
                        name='deviceName'
                        render={({ field }) => (
                          <InputText
                            label='Device name (optional)'
                            name='deviceName'
                            value={field.value}
                            onChange={field.onChange}
                            className='w-full'
                          />
                        )}
                      />
                      <div className='flex items-start gap-2 rounded-md border border-amber-500/30 bg-amber-500/10 px-3 py-2 text-xs text-amber-700 dark:text-amber-400'>
                        <ShieldCheck className='mt-0.5 h-4 w-4 shrink-0' />
                        <span>
                          Save your backup codes in a secure place — you'll need them if you lose
                          your device.
                        </span>
                      </div>
                    </Step>

                    <div className='flex flex-col gap-2'>
                      <Button
                        type='submit'
                        className='w-full rounded-lg py-5 text-sm'
                        disabled={!form.formState.isValid}
                      >
                        Enable two-factor authentication
                      </Button>
                      {handleCancel && (
                        <Button
                          type='button'
                          variant='outline'
                          className='w-full rounded-lg py-5 text-sm'
                          onClick={handleCancel}
                        >
                          Cancel
                        </Button>
                      )}
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

function Step({
  number,
  title,
  children,
}: {
  number: number
  title: string
  children: React.ReactNode
}) {
  return (
    <div className='space-y-3'>
      <div className='flex items-center gap-3'>
        <div className='flex h-6 w-6 shrink-0 items-center justify-center rounded-full bg-primary text-xs font-semibold text-primary-foreground'>
          {number}
        </div>
        <h2 className='text-sm font-semibold text-foreground'>{title}</h2>
      </div>
      <div className='space-y-3 sm:pl-9'>{children}</div>
    </div>
  )
}
