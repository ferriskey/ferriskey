import { useTranslation } from 'react-i18next'
import { Layers, Sparkles, Wrench } from 'lucide-react'

export const SIDEBAR_TAB_COMPONENTS = 'components'
export const SIDEBAR_TAB_PRESETS = 'presets'
export const SIDEBAR_TAB_TREE = 'tree'

export type SidebarTab =
  | typeof SIDEBAR_TAB_COMPONENTS
  | typeof SIDEBAR_TAB_PRESETS
  | typeof SIDEBAR_TAB_TREE

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
  const { t } = useTranslation('portal')

  return (
    <div className='flex items-center gap-1 overflow-x-auto scrollbar-none border-b border-border bg-muted/30 p-1'>
      <TabButton
        active={current === SIDEBAR_TAB_COMPONENTS}
        onClick={() => onChange(SIDEBAR_TAB_COMPONENTS)}
        icon={<Wrench size={13} />}
        label={t('builder.sidebar.components')}
      />
      <TabButton
        active={current === SIDEBAR_TAB_PRESETS}
        onClick={() => onChange(SIDEBAR_TAB_PRESETS)}
        icon={<Sparkles size={13} />}
        label={t('builder.sidebar.presets')}
      />
      <TabButton
        active={current === SIDEBAR_TAB_TREE}
        onClick={() => onChange(SIDEBAR_TAB_TREE)}
        icon={<Layers size={13} />}
        label={t('builder.sidebar.tree')}
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
      // `min-w-0` + `truncate` let a label give ground when the rail is narrow,
      // instead of the row overflowing into the container's hidden horizontal
      // scroll — where the third tab simply disappeared.
      className={`flex min-w-0 flex-1 items-center justify-center gap-1 rounded px-1.5 py-1 text-xs transition-colors ${
        active
          ? 'bg-background text-foreground shadow-sm'
          : 'text-muted-foreground hover:text-foreground'
      }`}
    >
      <span className='shrink-0'>{icon}</span>
      <span className='truncate'>{label}</span>
    </button>
  )
}
