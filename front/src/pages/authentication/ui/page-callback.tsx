import LoaderSpinner from '@/components/ui/loader-spinner'
import { XCircle } from 'lucide-react'

export interface PageCallbackProps {
  code?: string | null
  setup: boolean
  errorMessage?: string | null
}

export default function PageCallback({ code, setup, errorMessage }: PageCallbackProps) {
  if (errorMessage) {
    return <ErrorState message={errorMessage} />
  }

  if (setup && !code) {
    return <ErrorState message='Missing authorization code. Please try again.' />
  }

  return (
    <div className='flex min-h-svh items-center justify-center'>
      <LoaderSpinner />
    </div>
  )
}

function ErrorState({ message }: { message: string }) {
  return (
    <div className='flex min-h-svh items-center justify-center px-6'>
      <div className='w-full max-w-md rounded-md border border-destructive/30 bg-destructive/10 p-4 text-destructive'>
        <div className='flex gap-3'>
          <div className='shrink-0 pt-0.5'>
            <XCircle aria-hidden='true' className='size-5' />
          </div>
          <div className='space-y-1'>
            <h3 className='text-sm font-medium'>Unable to complete sign in</h3>
            <p className='text-sm'>{message}</p>
          </div>
        </div>
      </div>
    </div>
  )
}
