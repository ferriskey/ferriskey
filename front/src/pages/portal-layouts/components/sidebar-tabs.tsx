import { Layers, Sparkles, Wrench } from 'lucide-react'

export type SidebarTab = 'components' | 'presets' | 'tree'

interface Props {
  current: SidebarTab
  onChange: (tab: SidebarTab) => void
}

/**
 * Header at the top of the builder's left sidebar. Three modes:
 *  - Components: the drag-and-drop palette of individual blocks.
 *  - Presets: ready-made block trees the admin can stamp in one click (a
 *    full Sign-in card, an "Or continue with" group, …) — gets a new user
 *    to a working page in seconds instead of building from scratch.
 *  - Tree: hierarchical view of the current page/layout, useful when blocks
 *    are hidden in the canvas (e.g., display:none at the active breakpoint).
 */
export function SidebarTabs({ current, onChange }: Props) {
  return (
    <div className='flex items-center gap-1 overflow-x-auto scrollbar-none border-b border-border bg-muted/30 p-1'>
      <TabButton
        active={current === 'components'}
        onClick={() => onChange('components')}
        icon={<Wrench size={13} />}
        label='Components'
      />
      <TabButton
        active={current === 'presets'}
        onClick={() => onChange('presets')}
        icon={<Sparkles size={13} />}
        label='Presets'
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
      } whitespace-nowrap`}
    >
      {icon}
      {label}
    </button>
  )
}
