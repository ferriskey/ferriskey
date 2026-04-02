import { useLocation, useParams } from 'react-router'
import { Mail, ArrowLeft } from 'lucide-react'

export default function PageCheckYourEmail() {
  const { realm_name } = useParams()
  const location = useLocation()
  const email = (location.state as { email?: string })?.email

  return (
    <div className='flex flex-col items-center justify-center min-h-screen p-4'>
      <div className='max-w-md w-full space-y-6 text-center'>
        <div className='mx-auto w-16 h-16 bg-blue-100 rounded-full flex items-center justify-center'>
          <Mail className='w-8 h-8 text-blue-600' />
        </div>

        <h1 className='text-2xl font-bold'>Check your email</h1>

        <p className='text-muted-foreground'>
          We've sent a verification link to
          {email && <strong className='block mt-1'>{email}</strong>}
        </p>

        <p className='text-sm text-muted-foreground'>
          Click the link in the email to verify your account. The link expires in 24 hours.
        </p>

        <a
          href={`/realms/${realm_name}/authentication/login`}
          className='inline-flex items-center gap-2 text-sm text-primary hover:underline'
        >
          <ArrowLeft className='w-4 h-4' />
          Back to login
        </a>
      </div>
    </div>
  )
}
