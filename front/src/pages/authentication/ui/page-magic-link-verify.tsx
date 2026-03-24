import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import LoaderSpinner from '@/components/ui/loader-spinner'
import { Link } from 'react-router'

export interface PageMagicLinkVerifyProps {
  status: 'loading' | 'error'
  errorMessage?: string | null
}

export default function PageMagicLinkVerify({ status, errorMessage }: PageMagicLinkVerifyProps) {
  return (
    <div className='login-shell relative flex min-h-svh items-center justify-center px-6 py-10'>
      <div className='relative z-10 w-full max-w-sm md:max-w-md'>
        <Card className='overflow-hidden border p-0 shadow-sm'>
          <CardContent className='p-8 md:p-10'>
            <div className='flex flex-col items-center gap-6 text-center'>
              <div className='flex items-center gap-3'>
                <img src='/logo_ferriskey.png' alt='FerrisKey' className='h-7 w-7 object-contain' />
                <p className='text-xs font-semibold uppercase tracking-[0.35em] text-muted-foreground'>
                  FerrisKey
                </p>
              </div>

              {status === 'loading' && (
                <>
                  <LoaderSpinner />
                  <p className='text-sm text-muted-foreground'>Verifying your magic link...</p>
                </>
              )}

              {status === 'error' && (
                <>
                  <div className='rounded-md border border-destructive/30 bg-destructive/10 px-4 py-3 text-sm text-destructive'>
                    {errorMessage ?? 'This magic link is invalid or has expired.'}
                  </div>
                  <Button asChild variant='outline' className='w-full'>
                    <Link to={'../login'}>Back to login</Link>
                  </Button>
                </>
              )}
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  )
}
