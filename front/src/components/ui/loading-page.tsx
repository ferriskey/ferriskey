import { Skeleton } from './skeleton'

export default function LoadingPage() {
  return (
    <div className='w-full h-full flex flex-col gap-6'>
      <div className='p-6 bg-white'>
        <div className='flex flex-col gap-2'>
          <Skeleton className='w-64 h-6' />
          <Skeleton className='h-12 w-1/3' />
        </div>
      </div>

      <div className='p-6 bg-white'>
        <Skeleton className='h-[400px]' />
      </div>
    </div>
  )
}
