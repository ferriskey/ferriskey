import { cn } from '@/lib/utils'
import { PropsWithChildren, ReactNode } from 'react'

export interface BlockContentProps {
  title: string
  customWidth?: string
  className?: string
  classNameContent?: string
  dataTestId?: string
  headRight?: ReactNode
  headHeight?: string
}

export default function BlockContent({
  children,
  className = '',
  title,
  customWidth = 'w-full',
  classNameContent,
  dataTestId = 'block-content',
  headRight,
  headHeight
}: PropsWithChildren<BlockContentProps>) {

  return (
    <div
      data-testid={dataTestId}
      className={cn('mb-5 rounded border border-border bg-muted/30', customWidth, className)}
    >

      <div className={cn('flex items-center justify-between border-b border-border px-5', headHeight ? headHeight : 'h-9')}>
        <h2 className='text-sm font-medium text-muted-foreground'>{title}</h2>
        {headRight}
      </div>

      <div className={cn('p-5', classNameContent)}>{children}</div>
    </div>
  )
}
