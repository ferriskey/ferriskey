import { XCircle } from 'lucide-react'

export interface PageCallbackProps {
  code?: string | null
  setup: boolean
}

export default function PageCallback({ code, setup }: PageCallbackProps) {
  if (setup && !code) {
    return <ErrorCode />
  }

  return (
    <div>
      <h1>Page callback</h1>
      <p>{code}</p>
    </div>
  )
}

function ErrorCode() {
  return (
    <div className='rounded-md bg-red-50 p-4'>
      <div className='flex'>
        <div className='shrink-0'>
          <XCircle aria-hidden='true' className='size-5 text-red-400' />
        </div>
        <div className='ml-3'>
          <h3 className='text-sm font-medium text-red-800'>
            There were 2 errors with your submission
          </h3>
          <div className='mt-2 text-sm text-red-700'>
            <ul role='list' className='list-disc space-y-1 pl-5'>
              <li>Your password must be at least 8 characters</li>
              <li>
                Your password must include at least one pro wrestling finishing
                move
              </li>
            </ul>
          </div>
        </div>
      </div>
    </div>
  )
}
