import { LucideIcon } from 'lucide-react'
import { ReactNode } from 'react'

interface PageHeaderProps {
  icon: LucideIcon
  title: string
  description: string
  children?: ReactNode
}

export default function PageHeader({
  icon: Icon,
  title,
  description,
  children,
}: PageHeaderProps) {
  return (
    <div className='flex flex-col gap-4 border-b pb-6'>
      <div className='flex items-center gap-4'>
        <div className='flex h-12 w-12 items-center justify-center rounded-lg bg-primary/10'>
          <Icon className='h-6 w-6 text-primary' />
        </div>
        <div>
          <h1 className='text-3xl font-semibold tracking-tight'>{title}</h1>
          <p className='text-sm text-muted-foreground'>{description}</p>
        </div>
      </div>
      {children && <div className='flex justify-between items-center'>{children}</div>}
    </div>
  )
}
