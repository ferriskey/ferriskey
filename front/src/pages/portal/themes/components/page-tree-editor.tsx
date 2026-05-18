import {
  useCallback,
  useMemo,
  useState,
  type CSSProperties,
  type ReactNode,
} from 'react'
import { AlertTriangle, Monitor, Smartphone, Tablet } from 'lucide-react'
import { Badge } from '@/components/ui/badge'
import { ScrollArea } from '@/components/ui/scroll-area'
import {
  BuilderProvider,
  BuilderShell,
  Canvas,
  ConfigPanel,
  type BuilderNode,
} from '@/lib/builder-core'
import { createPortalAdapter, treeToReactNode } from '@/lib/builder-portal'
import type { Schemas } from '@/api/api.client'
import { useGetPortalPageRequirements } from '@/api/portal-theme.api'
import { cn } from '@/lib/utils'
import { PageComponentLibrary } from './page-component-library'

interface Props {
  realm: string
  pageType: Schemas.PortalPageType
  initialTree: unknown
  layoutTree: BuilderNode[] | null
  cssVars: CSSProperties
  /** Fired whenever the user edits the tree; parent batches saves under "Save theme". */
  onTreeChange: (tree: BuilderNode[]) => void
  /** Page-type / theme-tokens / layout nav rendered at the top of the left rail. */
  leftRailNav: ReactNode
}

type Viewport = 'desktop' | 'tablet' | 'mobile'

const VIEWPORT_WIDTHS: Record<Viewport, number | '100%'> = {
  desktop: '100%',
  tablet: 768,
  mobile: 375,
}

function parseTree(tree: unknown): BuilderNode[] {
  if (Array.isArray(tree)) return tree as BuilderNode[]
  if (tree && typeof tree === 'object' && Array.isArray((tree as { children?: unknown }).children)) {
    return (tree as { children: BuilderNode[] }).children
  }
  return []
}

function collectTypes(value: unknown, acc: Set<string>) {
  if (Array.isArray(value)) {
    value.forEach((v) => collectTypes(v, acc))
    return
  }
  if (value && typeof value === 'object') {
    const obj = value as Record<string, unknown>
    if (typeof obj.type === 'string') acc.add(obj.type)
    Object.values(obj).forEach((v) => collectTypes(v, acc))
  }
}

export default function PageTreeEditor({
  realm,
  pageType,
  initialTree,
  layoutTree,
  cssVars,
  onTreeChange,
  leftRailNav,
}: Props) {
  const adapter = useMemo(() => createPortalAdapter(), [])
  const [tree, setTree] = useState<BuilderNode[]>(() => parseTree(initialTree))
  const [viewport, setViewport] = useState<Viewport>('desktop')

  const { data: reqs } = useGetPortalPageRequirements({ realm })
  const requirements = useMemo(() => {
    const entry = reqs?.data?.find((r) => r.page_type === pageType)
    return entry?.required_blocks ?? []
  }, [reqs, pageType])

  const presentTypes = useMemo(() => {
    const acc = new Set<string>()
    collectTypes(tree, acc)
    return acc
  }, [tree])

  const missing = useMemo(
    () => requirements.filter((req) => !presentTypes.has(req)),
    [requirements, presentTypes],
  )

  const handleChange = useCallback(
    (next: BuilderNode[]) => {
      setTree(next)
      onTreeChange(next)
    },
    [onTreeChange],
  )

  const hasLayout = layoutTree && layoutTree.length > 0
  const viewportWidth = VIEWPORT_WIDTHS[viewport]
  // Numeric width is also passed to <Canvas /> so blocks know the renderable width.
  const canvasMaxWidth = typeof viewportWidth === 'number' ? viewportWidth : 1600

  return (
    <BuilderProvider adapter={adapter} initialTree={tree} onChange={handleChange}>
      <BuilderShell>
        <div className='grid h-full w-full min-w-0 grid-cols-[220px_1fr_320px] overflow-hidden'>
          <aside className='flex min-h-0 flex-col border-r border-border'>
            <ScrollArea className='h-full'>
              <div className='flex flex-col gap-3'>
                {leftRailNav}
                <PageComponentLibrary requiredTypes={requirements} />
              </div>
            </ScrollArea>
          </aside>

          <main className='flex min-w-0 flex-col overflow-hidden'>
            <header className='flex items-center justify-between border-b border-border px-4 py-2'>
              <div className='flex items-center gap-3'>
                <span className='text-sm font-medium capitalize'>
                  {pageType.replace(/_/g, ' ')} page
                </span>
                {missing.length === 0 ? (
                  <Badge variant='outline' className='gap-1 border-emerald-300 text-emerald-700'>
                    All required blocks present
                  </Badge>
                ) : (
                  <Badge variant='outline' className='gap-1 border-amber-300 text-amber-700'>
                    <AlertTriangle size={12} />
                    Missing: {missing.join(', ')}
                  </Badge>
                )}
              </div>
              <ViewportSwitcher viewport={viewport} onChange={setViewport} />
            </header>

            <div
              className='flex min-w-0 flex-1 justify-center overflow-auto bg-muted/30 p-6'
              style={cssVars}
            >
              <div
                className='shrink-0 self-start overflow-hidden rounded-lg border border-border bg-background shadow-sm transition-all duration-200'
                // `transform` traps `position: fixed` children inside the
                // preview frame instead of letting them escape to the viewport.
                // `containerType: size` lets us substitute vw/vh inside the
                // canvas with cqw/cqh so viewport-relative widths follow the
                // device frame, not the browser window.
                style={{
                  width: viewportWidth,
                  maxWidth: '100%',
                  transform: 'translate(0, 0)',
                  // `inline-size` only contains the width axis so the frame
                  // can still grow vertically with its content. `size` would
                  // collapse the frame to 0 height since we don't set one.
                  containerType: 'inline-size',
                  containerName: 'portal-preview',
                }}
              >
                {hasLayout
                  ? treeToReactNode(layoutTree!, {
                      pageContent: <Canvas maxWidth={canvasMaxWidth} />,
                    })
                  : <Canvas maxWidth={canvasMaxWidth} />}
              </div>
            </div>
          </main>

          <aside className='min-h-0 overflow-y-auto border-l border-border'>
            <ConfigPanel />
          </aside>
        </div>
      </BuilderShell>
    </BuilderProvider>
  )
}

function ViewportSwitcher({
  viewport,
  onChange,
}: {
  viewport: Viewport
  onChange: (v: Viewport) => void
}) {
  return (
    <div className='flex items-center gap-1 rounded-md border border-border p-0.5'>
      <ViewportButton
        active={viewport === 'desktop'}
        onClick={() => onChange('desktop')}
        label='Desktop'
      >
        <Monitor size={14} />
      </ViewportButton>
      <ViewportButton
        active={viewport === 'tablet'}
        onClick={() => onChange('tablet')}
        label='Tablet'
      >
        <Tablet size={14} />
      </ViewportButton>
      <ViewportButton
        active={viewport === 'mobile'}
        onClick={() => onChange('mobile')}
        label='Mobile'
      >
        <Smartphone size={14} />
      </ViewportButton>
    </div>
  )
}

function ViewportButton({
  active,
  onClick,
  label,
  children,
}: {
  active: boolean
  onClick: () => void
  label: string
  children: React.ReactNode
}) {
  return (
    <button
      type='button'
      title={label}
      onClick={onClick}
      className={cn(
        'rounded px-2 py-1 text-xs transition-colors',
        active ? 'bg-primary text-primary-foreground' : 'text-muted-foreground hover:bg-muted',
      )}
    >
      {children}
    </button>
  )
}
