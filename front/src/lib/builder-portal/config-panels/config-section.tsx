import { useState, type ReactNode } from 'react'
import { ChevronRight } from 'lucide-react'
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from '@/components/ui/collapsible'
import { cn } from '@/lib/utils'

interface ConfigSectionProps {
  title: string
  defaultOpen?: boolean
  children: ReactNode
}

export function ConfigSection({ title, defaultOpen = true, children }: ConfigSectionProps) {
  const [open, setOpen] = useState(defaultOpen)

  return (
    <Collapsible open={open} onOpenChange={setOpen}>
      <CollapsibleTrigger className='flex w-full items-center gap-1.5 py-1.5 text-xs font-semibold uppercase tracking-wide text-muted-foreground hover:text-foreground'>
        <ChevronRight className={cn('h-3.5 w-3.5 shrink-0 transition-transform', open && 'rotate-90')} />
        {title}
      </CollapsibleTrigger>
      <CollapsibleContent>
        <div className='flex flex-col gap-3 pb-3 pt-1.5'>{children}</div>
      </CollapsibleContent>
      <div className='h-px bg-border' />
    </Collapsible>
  )
}
