import { ReactNode } from 'react'
import { Button } from './button'
import { Plus, Upload } from 'lucide-react'

interface Tab {
  key: string
  label: string
  onClick: () => void
  active: boolean
}

interface OverviewHeaderProps {
  title: string
  description: string
  primaryAction?: {
    label: string
    onClick: () => void
  }
  secondaryAction?: {
    label: string
    onClick: () => void
    icon?: ReactNode
  }
  tabs: Tab[]
}

export function OverviewHeader({
  title,
  description,
  primaryAction,
  secondaryAction,
  tabs,
}: OverviewHeaderProps) {
  return (
    <>
      {/* Header */}
      <div className='-mx-8 -mt-8 px-8 pt-8 pb-4 border-b flex items-start justify-between gap-4'>
        <div>
          <h1 className='text-2xl font-bold tracking-tight'>{title}</h1>
          <p className='text-sm text-muted-foreground mt-1'>{description}</p>
        </div>
        <div className='flex items-center gap-2 shrink-0'>
          {primaryAction && (
            <Button onClick={primaryAction.onClick} size='sm'>
              <Plus className='h-4 w-4' />
              {primaryAction.label}
            </Button>
          )}
          {secondaryAction && (
            <Button variant='outline' size='sm' onClick={secondaryAction.onClick}>
              {secondaryAction.icon ?? <Upload className='h-4 w-4' />}
              {secondaryAction.label}
            </Button>
          )}
        </div>
      </div>

      {/* Tabs */}
      <div className='-mx-8 px-8 pb-4 border-b flex items-center gap-2 -mt-2'>
        {tabs.map((tab) => (
          <button
            key={tab.key}
            onClick={tab.onClick}
            className={`px-4 py-1.5 rounded-md text-sm font-medium transition-colors border ${
              tab.active
                ? 'bg-primary/10 text-primary border-primary/40'
                : 'bg-transparent text-foreground border-border hover:bg-muted'
            }`}
          >
            {tab.label}
          </button>
        ))}
      </div>
    </>
  )
}
