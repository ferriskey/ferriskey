import { ChevronDown } from 'lucide-react'
import { useState, type ReactNode } from 'react'
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from '@/components/ui/collapsible'
import { cn } from '@/lib/utils'

interface PanelSectionProps {
  title: string
  children: ReactNode
  /**
   * Collapsed sections start hidden — useful when a panel surfaces a
   * rarely-used group (e.g., link decoration in Typography). Defaults to
   * open so the panel is fully discoverable on first visit; the open/closed
   * state persists in component state across re-renders.
   */
  defaultOpen?: boolean
}

/**
 * Collapsible group inside a component-scoped panel (à la Webflow's
 * "Layout" / "Typography" foldouts in the style panel). Chevron rotates,
 * content animates open/closed. Sections are nested inside panels and
 * remember their own open state.
 */
export function PanelSection({ title, children, defaultOpen = true }: PanelSectionProps) {
  const [open, setOpen] = useState(defaultOpen)
  return (
    <Collapsible open={open} onOpenChange={setOpen} className='border-b border-border'>
      {/* Sticky header so the section title stays visible while scrolling
          through a long panel — mirrors Webflow's style-panel behaviour
          where you always know which group of properties you're editing.
          `z-10` keeps it above content; the matching `bg-background` solid
          fill prevents the rows underneath from bleeding through. */}
      <CollapsibleTrigger
        className={cn(
          'sticky top-0 z-10 flex w-full items-center justify-between bg-background py-2 text-left',
        )}
      >
        <span className='text-[11px] font-semibold uppercase tracking-wide text-muted-foreground'>
          {title}
        </span>
        <ChevronDown
          className={cn('h-3 w-3 text-muted-foreground transition-transform', open && 'rotate-180')}
        />
      </CollapsibleTrigger>
      <CollapsibleContent className='pb-3'>
        <div className='flex flex-col gap-1.5'>{children}</div>
      </CollapsibleContent>
    </Collapsible>
  )
}

/**
 * Header rendered at the top of every panel. Uniform title + subtitle so the
 * five panels feel like a family.
 */
export function PanelHeader({
  title,
  description,
}: {
  title: string
  description: string
}) {
  return (
    <div className='pb-2'>
      <h2 className='text-sm font-semibold'>{title}</h2>
      <p className='text-xs text-muted-foreground'>{description}</p>
    </div>
  )
}
