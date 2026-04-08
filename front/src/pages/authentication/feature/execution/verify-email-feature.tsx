import { Mail } from 'lucide-react'

export default function VerifyEmailFeature() {
  return (
    <div className='flex flex-col items-center justify-center min-h-screen p-4'>
      <div className='max-w-md w-full space-y-6 text-center'>
        <div className='mx-auto w-16 h-16 bg-blue-100 rounded-full flex items-center justify-center'>
          <Mail className='w-8 h-8 text-blue-600' />
        </div>

        <h1 className='text-2xl font-bold'>Verify your email</h1>

        <p className='text-muted-foreground'>
          A verification link has been sent to your email address. Please check your inbox and click
          the link to verify your account.
        </p>

        <p className='text-sm text-muted-foreground'>
          If you don't see the email, check your spam folder.
        </p>
      </div>
    </div>
  )
}
