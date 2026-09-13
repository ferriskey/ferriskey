import type { ReactNode } from 'react'
import { ArrowLeft } from 'lucide-react'
import { Button } from '@/components/kit/button'
import { tokens } from '@/styles/style-tokens'

export interface DetailHeaderProps {
  onBack?: () => void
  backLabel?: string
  icon?: ReactNode
  title?: ReactNode
  caption?: ReactNode
  pills?: ReactNode
  meta?: ReactNode
}

export function DetailHeader({
  onBack,
  backLabel,
  icon,
  title,
  caption,
  pills,
  meta,
}: DetailHeaderProps) {
  return (
    <>
      {onBack && backLabel && (
        <Button
          variant='ghost'
          size='sm'
          className='-ml-2 mb-2 text-neutral-500 dark:text-neutral-400'
          onClick={onBack}
        >
          <ArrowLeft className='size-3.5' />
          {backLabel}
        </Button>
      )}

      <div className='flex flex-wrap items-start justify-between gap-4'>
        <div className='flex items-center gap-3'>
          {icon}
          <div className='min-w-0'>
            <h1 className={tokens.header.title}>{title}</h1>
            {caption}
            {pills && <div className='mt-1.5 flex flex-wrap items-center gap-2'>{pills}</div>}
          </div>
        </div>

        {meta}
      </div>
    </>
  )
}
