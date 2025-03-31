import { LoginForm } from '@/components/login-form'
import {
  Card,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card'
import { cn } from '@/lib/utils'
import { GalleryVerticalEnd } from 'lucide-react'

export default function PageLogin() {
  return (
    <div className='flex min-h-svh flex-col items-center justify-center gap-6 bg-muted p-6 md:p-10'>
      <div className='flex w-full max-w-sm flex-col gap-6'>
        <a href='#' className='flex items-center gap-2 self-center font-medium'>
          <div className='flex h-10 w-10 border rounded-md overflow-hidden items-center justify-center bg-primary text-primary-foreground'>
            {/* <GalleryVerticalEnd className='size-4' /> */}
            <img src='/logo_ferriskey.png' alt='' />
          </div>
          FerrisKey
        </a>
        <LoginForm />
      </div>
    </div>
  )
}
