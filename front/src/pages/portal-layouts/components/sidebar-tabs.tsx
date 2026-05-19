import { Layers, Wrench } from 'lucide-react'

export type SidebarTab = 'components' | 'tree'

interface Props {
  current: SidebarTab
  onChange: (tab: SidebarTab) => void
}

/**
 * Header at the top of the builder's left sidebar that swaps between the
 * drag-and-drop component palette and the hierarchical tree view of the
 * current layout / page. The tree exists so the author can reach blocks
 * that are hidden in the canvas (e.g. `display: none` at the active
 * breakpoint, or stacked under siblings).
 */
export function SidebarTabs({ current, onChange }: Props) {
  return (
    <div className='flex items-center gap-1 border-b border-border bg-muted/30 p-1'>
      <TabButton
        active={current === 'components'}
        onClick={() => onChange('components')}
        icon={<Wrench size={13} />}
        label='Components'
      />
      <TabButton
        active={current === 'tree'}
        onClick={() => onChange('tree')}
        icon={<Layers size={13} />}
        label='Tree'
      />
    </div>
  )
}

function TabButton({
  active,
  onClick,
  icon,
  label,
}: {
  active: boolean
  onClick: () => void
  icon: React.ReactNode
  label: string
}) {
  return (
    <button
      type='button'
      onClick={onClick}
      className={`flex flex-1 items-center justify-center gap-1.5 rounded px-2 py-1 text-xs transition-colors ${
        active
          ? 'bg-background text-foreground shadow-sm'
          : 'text-muted-foreground hover:text-foreground'
      }`}
    >
      {icon}
      {label}
    </button>
  )
}
