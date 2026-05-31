import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import { FormField } from '@/components/ui/form'
import { InputText } from '@/components/ui/input-text'
import { UseFormReturn } from 'react-hook-form'
import { Link, useParams } from 'react-router'
import { MagicLinkSchema } from '@/pages/authentication/schemas/magic-link.schema'
import { ArrowLeft, Mail } from 'lucide-react'
import './page-login.css'

export interface PageMagicLinkRequestProps {
  form: UseFormReturn<MagicLinkSchema>
  onSubmit: (data: MagicLinkSchema) => void
  /**
   * `true` once the API has accepted the request and the user should now go
   * check their inbox. Drives the success view that replaces the form.
   */
  sent: boolean
  isPending: boolean
}

/**
 * Dedicated "request a magic link" page. Reached from the magic-link button
 * inside the customizable portal tree (`data-fk-action="magic-link"`), which
 * navigates here instead of trying to collect the email inline — so the user
 * always lands on a focused, full-page form regardless of how the realm's
 * login screen was authored.
 */
export default function PageMagicLinkRequest({
  form,
  onSubmit,
  sent,
  isPending,
}: PageMagicLinkRequestProps) {
  const { realm_name } = useParams()

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
                      {sent ? 'Check your inbox' : 'Sign in by email'}
                    </h1>
                  </div>

                  {sent ? (
                    <div className='flex flex-col items-center gap-4 py-4'>
                      <div className='flex h-14 w-14 items-center justify-center rounded-full bg-primary/10'>
                        <Mail className='h-7 w-7 text-primary' />
                      </div>
                      <p className='text-center text-sm text-muted-foreground'>
                        We sent a magic link to your email address. Click the link to sign in — no password needed.
                      </p>
                      <p className='text-center text-xs text-muted-foreground'>
                        The link expires in 15 minutes. Check your spam folder if you don&apos;t see it.
                      </p>
                    </div>
                  ) : (
                    <form onSubmit={form.handleSubmit(onSubmit)}>
                      <div className='flex flex-col gap-7'>
                        <p className='text-sm text-muted-foreground'>
                          Enter your email address and we&apos;ll send you a link to sign in instantly.
                        </p>
                        <div className='grid gap-3'>
                          <FormField
                            control={form.control}
                            name='email'
                            render={({ field }) => (
                              <InputText
                                {...field}
                                label='Email address'
                                name='email'
                                type='email'
                                required
                                className='w-full'
                                autoComplete='email'
                                error={form.formState.errors.email?.message}
                              />
                            )}
                          />
                        </div>
                        <Button type='submit' className='w-full rounded-lg py-5 text-sm' disabled={isPending}>
                          {isPending ? 'Sending...' : 'Send magic link'}
                        </Button>
                      </div>
                    </form>
                  )}

                  <div className='text-center text-xs text-muted-foreground md:text-sm'>
                    <Link
                      to={`/realms/${realm_name}/authentication/login`}
                      className='inline-flex items-center gap-1 font-semibold text-foreground underline underline-offset-4'
                    >
                      <ArrowLeft className='h-3 w-3' />
                      Back to login
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
