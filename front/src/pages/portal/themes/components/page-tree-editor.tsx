import {
  useCallback,
  useEffect,
  useMemo,
  useRef,
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
  SelectionBreadcrumb,
  useEditingBreakpoint,
  useSelectedNode,
  type Breakpoint,
  type BuilderNode,
} from '@/lib/builder-core'
import {
  createPortalAdapter,
  generateBreakpointCss,
  treeToReactNode,
  useIframeFit,
} from '@/lib/builder-portal'
import { CanvasFrame } from '@/lib/builder-portal/components/canvas-frame'
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

// Device presets sized to real hardware so the previews actually feel like
// the device they're labelled — matched to Firefox's device-inspector
// profiles for consistency with the in-browser DevTools "Responsive Design
// Mode". Tailwind's responsive thresholds still drive the cascade:
//   iphone  402×874   — below sm (640), so only the BASE layer applies.
//   tablet  768×1024  — md+ kicks in (sm fires too, since 768 ≥ 640).
//   desktop 1280×800  — xl+ kicks in.
type Viewport = 'iphone' | 'tablet' | 'desktop'

const VIEWPORT_WIDTHS: Record<Viewport, number> = {
  iphone: 402,
  tablet: 768,
  desktop: 1280,
}

const VIEWPORT_HEIGHTS: Record<Viewport, number> = {
  iphone: 874,
  tablet: 1024,
  desktop: 800,
}

// Smallest device whose width activates a given Tailwind breakpoint —
// clicking a bp tab in the config panel switches the preview to this device
// so the user immediately sees the layer they're editing. `sm` (640) is now
// previewed on the tablet (768) since the iphone preset sits below the sm
// threshold; tablet still fires `sm:` utilities.
const BREAKPOINT_TO_VIEWPORT: Record<Breakpoint, Viewport> = {
  sm: 'tablet',
  md: 'tablet',
  lg: 'desktop',
  xl: 'desktop',
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
  const iframeRectRef = useRef<DOMRect | null>(null)
  const iframeScaleRef = useRef<number>(1)
  const getIframeRect = useCallback(() => iframeRectRef.current, [])
  const getIframeScale = useCallback(() => iframeScaleRef.current, [])
  // Stable callback so the bridge below only fires on real bp changes,
  // not on every parent re-render.
  const handleBreakpointChange = useCallback((bp: Breakpoint | null) => {
    if (bp) setViewport(BREAKPOINT_TO_VIEWPORT[bp])
  }, [])

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
  const viewportHeight = VIEWPORT_HEIGHTS[viewport]
  // Numeric width is also passed to <Canvas /> so blocks know the renderable width.
  const canvasMaxWidth = typeof viewportWidth === 'number' ? viewportWidth : 1600

  // Shrink the iframe to fit the canvas area when the device preset would
  // otherwise overflow. The iframe's internal viewport stays at the device
  // size so `@media` queries keep firing as expected.
  const { containerRef: canvasAreaRef, scale: iframeScale } = useIframeFit({
    width: viewportWidth,
    height: viewportHeight,
    padding: 20, // matches the `p-5` padding on the canvas area
  })
  // Keep the ref in sync so BuilderShell's collision detection picks up the
  // current scale on every drag check.
  useEffect(() => {
    iframeScaleRef.current = iframeScale
  }, [iframeScale])

  return (
    <BuilderProvider adapter={adapter} initialTree={tree} onChange={handleChange}>
      <BreakpointToDeviceSync onBreakpointChange={handleBreakpointChange} />
      <BuilderShell getIframeRect={getIframeRect} getIframeScale={getIframeScale}>
        <EditorGrid>
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
            <div className='border-b border-border bg-muted/30'>
              <SelectionBreadcrumb />
            </div>

            <div
              ref={canvasAreaRef}
              className='flex min-w-0 flex-1 items-start justify-center overflow-hidden p-5'
              style={{
                backgroundColor: '#f8f9fa',
                backgroundImage:
                  'radial-gradient(circle, #d1d5db 1px, transparent 1px)',
                backgroundSize: '20px 20px',
              }}
            >
              {/* Outer box reserves the *scaled* visual footprint so the
                  iframe centers correctly and never pushes the layout into a
                  scroll. The inner `<CanvasFrame>` keeps its device-size
                  width/height; we shrink it visually via `transform: scale`. */}
              <div
                className='flex-shrink-0 self-start overflow-hidden rounded-lg border border-border bg-background shadow-sm transition-all duration-200'
                style={{
                  width: viewportWidth * iframeScale,
                  height: viewportHeight * iframeScale,
                }}
              >
                <div
                  style={{
                    width: viewportWidth,
                    height: viewportHeight,
                    transform: `scale(${iframeScale})`,
                    transformOrigin: 'top left',
                  }}
                >
                  <CanvasFrame
                    width={viewportWidth}
                    height={viewportHeight}
                    cssVars={cssVars}
                    responsiveCss={[
                      generateBreakpointCss(tree),
                      layoutTree ? generateBreakpointCss(layoutTree) : '',
                    ]
                      .filter(Boolean)
                      .join('\n')}
                    onRectChange={(rect) => {
                      iframeRectRef.current = rect
                    }}
                  >
                    {hasLayout
                      ? treeToReactNode(layoutTree!, {
                          // Inline mode: the layout's `<page-content>` slot
                          // controls vertical alignment, so the canvas
                          // should not stretch — mirrors how the runtime
                          // injects a `<form>` block here.
                          pageContent: (
                            <Canvas maxWidth={canvasMaxWidth} fillHeight={false} />
                          ),
                        })
                      : <Canvas maxWidth={canvasMaxWidth} />}
                  </CanvasFrame>
                </div>
              </div>
            </div>
          </main>

          <ConditionalConfigSidebar />
        </EditorGrid>
      </BuilderShell>
    </BuilderProvider>
  )
}

/**
 * Two-or-three-column grid driven by selection: the right config rail only
 * occupies space while a node is selected, so the canvas reclaims it whenever
 * the user clicks away. Must live inside `BuilderProvider`.
 */
function EditorGrid({ children }: { children: ReactNode }) {
  const selected = useSelectedNode()
  return (
    <div
      className='grid h-full w-full min-w-0 overflow-hidden'
      style={{
        gridTemplateColumns: selected ? '220px 1fr 320px' : '220px 1fr',
      }}
    >
      {children}
    </div>
  )
}

function ConditionalConfigSidebar() {
  const selected = useSelectedNode()
  if (!selected) return null
  return (
    <aside className='min-h-0 overflow-y-auto border-l border-border'>
      <ConfigPanel />
    </aside>
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
        active={viewport === 'iphone'}
        onClick={() => onChange('iphone')}
        label='iPhone — 402 (base, below Tailwind sm)'
      >
        <Smartphone size={14} />
      </ViewportButton>
      <ViewportButton
        active={viewport === 'tablet'}
        onClick={() => onChange('tablet')}
        label='Tablet — 768 (Tailwind md)'
      >
        <Tablet size={14} />
      </ViewportButton>
      <ViewportButton
        active={viewport === 'desktop'}
        onClick={() => onChange('desktop')}
        label='Desktop — 1280 (Tailwind xl)'
      >
        <Monitor size={14} />
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

/**
 * Bridges the BreakpointContext (set by the config panel's bp tabs) with the
 * page tree editor's device viewport state. When the user clicks `md`, the
 * preview jumps to Tablet so the layer they're editing is actually visible.
 */
function BreakpointToDeviceSync({
  onBreakpointChange,
}: {
  onBreakpointChange: (bp: Breakpoint | null) => void
}) {
  const { current } = useEditingBreakpoint()
  useEffect(() => {
    onBreakpointChange(current)
  }, [current, onBreakpointChange])
  return null
}
