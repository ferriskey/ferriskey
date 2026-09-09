import { ArrowUpRight, Construction } from 'lucide-react'
import { Link, useParams } from 'react-router-dom'
import { Button } from '@/components/kit/button'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'
import { RouterParams } from '@/routes/router'
import { REALM_URL } from '@/routes/router'

export default function NotMigratedYet({ domain }: { domain: string }) {
  const { realm_name = 'master' } = useParams<RouterParams>()

  return (
    <div className={cn('mx-auto', tokens.page.maxWidth, tokens.page.padding)}>
      <div className={cn(tokens.surface.panel, 'grid place-items-center px-6 py-20')}>
        <Construction className='size-6 text-neutral-300' strokeWidth={1.5} />
        <p className='mt-3 text-sm font-medium text-neutral-700'>
          Not migrated yet
        </p>
        <p className='mt-1 max-w-sm text-center text-sm text-neutral-500'>
          This screen still lives in the current console. It will land here as the
          migration progresses.
        </p>
        <Button asChild variant='outline' size='sm' className='mt-4'>
          <Link to={`${REALM_URL(realm_name)}/${domain}`}>
            Open in the current console
            <ArrowUpRight />
          </Link>
        </Button>
      </div>
    </div>
  )
}
