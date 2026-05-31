import { CheckCircle } from 'lucide-react'
import { Link, useParams } from 'react-router'
import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import '../ui/page-login.css'

/**
 * Post-verification success screen. Reached via a redirect from
 * `/verify-email?token=…` once the backend has actually confirmed the
 * email. Kept as the React fallback when the realm admin hasn't built a
 * custom `email_verified` tree in the theme builder — when they have,
 * `<PortalLayoutWrapper>` renders the custom tree and ignores this
 * component.
 */
export default function PageEmailVerifiedFeature() {
  const { realm_name } = useParams()

  return (
    <div className='login-shell relative flex min-h-svh items-center justify-center px-6 py-10'>
      <div className='relative z-10 w-full max-w-sm md:max-w-md lg:max-w-lg'>
        <div className='flex flex-col gap-6'>
          <Card className='login-card overflow-hidden border p-0 shadow-sm'>
            <CardContent className='grid gap-0 p-0'>
              <div className='p-8 md:p-10'>
                <div className='flex flex-col items-center gap-6 text-center'>
                  <div className='flex h-14 w-14 items-center justify-center rounded-full bg-green-100'>
                    <CheckCircle className='h-8 w-8 text-green-600' />
                  </div>
                  <div className='space-y-2'>
                    <h1 className='text-2xl font-semibold tracking-tight'>
                      Email verified
                    </h1>
                    <p className='text-sm text-muted-foreground'>
                      Your email address has been confirmed. You can now sign in
                      to your account.
                    </p>
                  </div>
                  <Button asChild className='w-full rounded-lg py-5 text-sm'>
                    <Link to={`/realms/${realm_name}/authentication/login`}>
                      Continue to sign in
                    </Link>
                  </Button>
                </div>
              </div>
            </CardContent>
          </Card>
        </div>
      </div>
    </div>
  )
}
