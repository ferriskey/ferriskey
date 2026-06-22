import { useMemo, useState, type CSSProperties } from 'react'
import { ArrowLeft, CheckCircle2, LayoutTemplate, Palette, Save } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Label } from '@/components/ui/label'
import { ScrollArea } from '@/components/ui/scroll-area'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { cn } from '@/lib/utils'
import type { Schemas } from '@/api/api.client'
import type { BuilderNode } from '@/lib/builder-core'
import { ButtonsPanel } from '@/pages/portal-theme/components/panels/buttons-panel'
import { InputsPanel } from '@/pages/portal-theme/components/panels/inputs-panel'
import { WidgetPanel } from '@/pages/portal-theme/components/panels/widget-panel'
import { TypographyPanel } from '@/pages/portal-theme/components/panels/typography-panel'
import { PagePanel } from '@/pages/portal-theme/components/panels/page-panel'
import { PreviewCard } from '@/pages/portal-theme/components/preview-card'
import { usePortalThemeContext } from '@/pages/portal-theme/context/portal-theme-context'
import { themeToCssVars } from '@/pages/portal-theme/lib/theme'
import type { BuilderTab } from '../feature/page-theme-builder-feature'
import PageTreeEditor from '../components/page-tree-editor'

type PageType = Schemas.PortalPageType

const PAGE_TYPES: { id: PageType; label: string }[] = [
  { id: 'login', label: 'Login' },
  { id: 'register', label: 'Register' },
  { id: 'totp', label: 'TOTP' },
  { id: 'forgot_password', label: 'Forgot password' },
  { id: 'reset_password', label: 'Reset password' },
  { id: 'magic_link_verify', label: 'Magic link verify' },
  { id: 'magic_link_request', label: 'Magic link request' },
  { id: 'verify_email', label: 'Verify email' },
  { id: 'email_verified', label: 'Email verified' },
  { id: 'totp_setup', label: 'TOTP setup' },
]

interface Props {
  theme: Schemas.PortalTheme
  layouts: Schemas.PortalLayout[]
  isSavingMetadata: boolean
  isSavingPage: boolean
  isActivating: boolean
  realm: string
  activeTab: BuilderTab
  onBack: () => void
  onTabChange: (tab: BuilderTab) => void
  onSaveTheme: (
    name: string,
    layoutId: string | null,
    config: object,
    pages: { pageType: PageType; tree: unknown }[]
  ) => void
  onActivate: () => void
}

export default function PageThemeBuilder({
  theme,
  layouts,
  isSavingMetadata,
  isSavingPage,
  isActivating,
  realm,
  activeTab,
  onBack,
  onTabChange,
  onSaveTheme,
  onActivate,
}: Props) {
  const [name, setName] = useState(theme.name)
  const [layoutId, setLayoutId] = useState<string | null>(theme.layout_id ?? null)
  const [pageOverrides, setPageOverrides] = useState<Partial<Record<PageType, BuilderNode[]>>>({})
  const { theme: tokens } = usePortalThemeContext()

  const cssVars = useMemo(() => themeToCssVars(tokens) as CSSProperties, [tokens])

  const layoutTree = useMemo<BuilderNode[] | null>(() => {
    if (!layoutId) return null
    const selected = layouts.find((l) => l.id === layoutId)
    if (!selected) return null
    return parseLayoutTree(selected.tree)
  }, [layoutId, layouts])

  return (
    <div className='flex h-[calc(100vh-3rem)] w-full min-w-0 flex-col overflow-hidden'>
      <header className='flex items-center gap-3 border-b border-border px-4 py-2'>
        <Button variant='ghost' size='icon' onClick={onBack}>
          <ArrowLeft size={16} />
        </Button>
        <input
          type='text'
          className='flex-1 bg-transparent text-lg font-medium outline-none placeholder:text-muted-foreground'
          placeholder='Theme name…'
          value={name}
          onChange={(e) => setName(e.target.value)}
        />
        <Button variant='outline' onClick={onActivate} disabled={isActivating}>
          <CheckCircle2 size={16} />
          {isActivating ? 'Activating…' : 'Activate'}
        </Button>
        <Button
          onClick={() => {
            const pages = Object.entries(pageOverrides)
              .filter(([, tree]) => tree)
              .map(([pageType, tree]) => ({
                pageType: pageType as PageType,
                tree: tree as BuilderNode[],
              }))
            onSaveTheme(name, layoutId, tokens, pages)
          }}
          disabled={isSavingMetadata || isSavingPage}
        >
          <Save size={16} />
          {isSavingMetadata || isSavingPage ? 'Saving…' : 'Save theme'}
        </Button>
      </header>

      {(() => {
        const isSectionActive = (s: 'theme' | 'layout') =>
          activeTab.kind === 'section' && activeTab.section === s
        const isPageActive = (p: PageType) => activeTab.kind === 'page' && activeTab.pageType === p

        const nav = (
          <nav className='flex flex-col gap-1 p-2'>
            <NavButton
              active={isSectionActive('theme')}
              onClick={() => onTabChange({ kind: 'section', section: 'theme' })}
              icon={<Palette size={14} />}
              label='Theme'
            />
            <NavButton
              active={isSectionActive('layout')}
              onClick={() => onTabChange({ kind: 'section', section: 'layout' })}
              icon={<LayoutTemplate size={14} />}
              label='Layout'
            />
            <div className='mt-2 px-3 pb-1 text-[10px] font-semibold uppercase tracking-wider text-muted-foreground'>
              Pages
            </div>
            {PAGE_TYPES.map(({ id, label }) => (
              <NavButton
                key={id}
                active={isPageActive(id)}
                onClick={() => onTabChange({ kind: 'page', pageType: id })}
                label={label}
              />
            ))}
          </nav>
        )

        if (activeTab.kind === 'page') {
          // Page tree editor owns its own grid (rail + main + config); we
          // pass the nav so the same left column holds both rails.
          return (
            <PageTreeEditor
              key={activeTab.pageType}
              realm={realm}
              pageType={activeTab.pageType}
              initialTree={readPageTree(theme, activeTab.pageType)}
              layoutTree={layoutTree}
              cssVars={cssVars}
              onTreeChange={(tree) =>
                setPageOverrides((prev) => ({ ...prev, [activeTab.pageType]: tree }))
              }
              leftRailNav={nav}
            />
          )
        }

        return (
          <div className='grid flex-1 grid-cols-[220px_1fr] overflow-hidden'>
            <aside className='border-r border-border'>
              <ScrollArea className='h-full'>{nav}</ScrollArea>
            </aside>
            <main className='overflow-hidden'>
              {activeTab.section === 'theme' && <ThemeTokensTab cssVars={cssVars} />}
              {activeTab.section === 'layout' && (
                <LayoutTab layouts={layouts} layoutId={layoutId} onChange={setLayoutId} />
              )}
            </main>
          </div>
        )
      })()}
    </div>
  )
}

function ThemeTokensTab({ cssVars }: { cssVars: CSSProperties }) {
  // `cssVars` are now applied inside `PreviewCard` (via the context), so the
  // outer div only needs the muted background. We keep the prop for API
  // stability and apply it to the page-bg surface so tweaks to the page
  // background token are still visible behind the widget.
  return (
    <div className='grid h-full grid-cols-[1fr_360px] overflow-hidden'>
      <div className='min-h-0 overflow-hidden bg-muted/30' style={cssVars}>
        <PreviewCard />
      </div>
      {/* `min-h-0` lets the grid child shrink to the row height so the
          ScrollArea inside can actually clip and scroll. Without it, the
          aside grows to fit its content and the ScrollArea inherits an
          infinite track, which renders as "no scroll". */}
      <aside className='min-h-0 border-l border-border'>
        <ScrollArea className='h-full'>
          <div className='flex flex-col gap-6 p-4'>
            <ButtonsPanel />
            <InputsPanel />
            <WidgetPanel />
            <TypographyPanel />
            <PagePanel />
          </div>
        </ScrollArea>
      </aside>
    </div>
  )
}

function LayoutTab({
  layouts,
  layoutId,
  onChange,
}: {
  layouts: Schemas.PortalLayout[]
  layoutId: string | null
  onChange: (id: string | null) => void
}) {
  return (
    <div className='flex max-w-xl flex-col gap-4 p-6'>
      <div>
        <h3 className='text-lg font-semibold'>Layout</h3>
        <p className='text-sm text-muted-foreground'>
          Choose the layout (header / footer / sidebar wrapper) used by every page of this theme.
          The layouts list lives in the “Layouts” tab.
        </p>
      </div>
      <div className='flex flex-col gap-2'>
        <Label className='text-xs'>Selected layout</Label>
        <Select
          value={layoutId ?? 'none'}
          onValueChange={(value) => onChange(value === 'none' ? null : value)}
        >
          <SelectTrigger>
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value='none'>None — render page content bare</SelectItem>
            {layouts.map((l) => (
              <SelectItem key={l.id} value={l.id}>
                {l.name}
                {l.is_default ? ' (default)' : ''}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
        <p className='text-xs text-muted-foreground'>
          Don't forget to hit “Save theme” at the top after changing the layout.
        </p>
      </div>
    </div>
  )
}

function NavButton({
  active,
  onClick,
  icon,
  label,
}: {
  active: boolean
  onClick: () => void
  icon?: React.ReactNode
  label: string
}) {
  return (
    <button
      type='button'
      onClick={onClick}
      className={cn(
        'flex items-center gap-2 rounded-md px-3 py-2 text-sm transition-colors',
        active ? 'bg-sidebar-primary/15 text-sidebar-primary' : 'hover:bg-muted'
      )}
    >
      {icon}
      <span>{label}</span>
    </button>
  )
}

function readPageTree(theme: Schemas.PortalTheme, pageType: PageType): unknown {
  // The Rust `PortalThemePages` struct is serialised with
  // `#[serde(rename_all = "camelCase")]`, so the JSON keys are
  // `forgotPassword` / `magicLinkRequest` / `verifyEmail` / etc. But the
  // `PortalPageType` enum is serialised in snake_case, so `pageType` is
  // `'forgot_password'`, `'magic_link_request'`, … — a direct
  // `pages[pageType]` lookup misses every multi-word page and returns an
  // empty array, which is why the builder canvas stays blank on refresh
  // for those pages. Convert the enum value to camelCase before reading.
  // Single-word pages (login / register / totp) are unaffected (snake ≡
  // camel for those), which is why the bug only surfaced once we shipped
  // the multi-word ones.
  const pages = theme.pages as Record<string, unknown> | undefined
  return pages?.[snakeToCamel(pageType)] ?? []
}

function snakeToCamel(value: string): string {
  return value.replace(/_([a-z])/g, (_, c: string) => c.toUpperCase())
}

function parseLayoutTree(tree: unknown): BuilderNode[] {
  if (Array.isArray(tree)) return tree as BuilderNode[]
  if (
    tree &&
    typeof tree === 'object' &&
    Array.isArray((tree as { children?: unknown }).children)
  ) {
    return (tree as { children: BuilderNode[] }).children
  }
  return []
}
