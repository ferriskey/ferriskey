import { Button } from '@/components/ui/button'
import {
  BuilderProvider,
  BuilderShell,
  Canvas,
  ConfigPanel,
  SelectionBreadcrumb,
  useEditingBreakpoint,
  useSelectedNode,
  type Breakpoint,
  type BuilderAdapter,
  type BuilderNode,
} from '@/lib/builder-core'
import { CanvasFrame } from '@/lib/builder-portal/components/canvas-frame'
import { generateBreakpointCss, useIframeFit } from '@/lib/builder-portal'
import { LayoutComponentLibrary } from '../components/layout-component-library'
import { ArrowLeft, Monitor, Save, Smartphone, Tablet } from 'lucide-react'
import { useCallback, useEffect, useRef, useState, type CSSProperties } from 'react'

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

interface Props {
  adapter: BuilderAdapter
  tree: BuilderNode[]
  onTreeChange: (tree: BuilderNode[]) => void
  name: string
  onNameChange: (name: string) => void
  isNew: boolean
  isSaving: boolean
  cssVars: CSSProperties
  onSave: () => void
  onBack: () => void
}

export default function PagePortalLayoutBuilder({
  adapter,
  tree,
  onTreeChange,
  name,
  onNameChange,
  isNew,
  isSaving,
  cssVars,
  onSave,
  onBack,
}: Props) {
  const [viewport, setViewport] = useState<Viewport>('desktop')
  const iframeRectRef = useRef<DOMRect | null>(null)
  const iframeScaleRef = useRef<number>(1)
  const getIframeRect = useCallback(() => iframeRectRef.current, [])
  const getIframeScale = useCallback(() => iframeScaleRef.current, [])
  // Stable callback identity so the bridge below only fires when the editing
  // bp actually changes — not on every parent re-render (which would snap
  // the viewport back and override the user's manual device choice).
  const handleBreakpointChange = useCallback((bp: Breakpoint | null) => {
    if (bp) setViewport(BREAKPOINT_TO_VIEWPORT[bp])
  }, [])

  const viewportWidth = VIEWPORT_WIDTHS[viewport]
  const viewportHeight = VIEWPORT_HEIGHTS[viewport]
  // Shrink the iframe to fit the available canvas area; keeps the iframe's
  // internal viewport at the device size so `@media` rules still apply.
  const { containerRef: canvasAreaRef, scale: iframeScale } = useIframeFit({
    width: viewportWidth,
    height: viewportHeight,
    padding: 20,
  })
  useEffect(() => {
    iframeScaleRef.current = iframeScale
  }, [iframeScale])

  return (
    <BuilderProvider adapter={adapter} initialTree={tree} onChange={onTreeChange}>
      <BreakpointToDeviceSync onBreakpointChange={handleBreakpointChange} />
      {/* `h-full` plays off the PortalShell's `min-h-0 flex-1 overflow-hidden`
          wrapper: the builder fills exactly what's left between the TopBar
          and the Portail sub-nav, no page-level scroll. */}
      <div className='flex h-full w-full min-w-0 flex-col overflow-hidden'>
        <header className='flex items-center gap-3 border-b border-border px-4 py-2'>
          <Button variant='ghost' size='icon' onClick={onBack}>
            <ArrowLeft size={16} />
          </Button>

          <input
            type='text'
            className='flex-1 bg-transparent text-lg font-medium outline-none placeholder:text-muted-foreground'
            placeholder='Layout name...'
            value={name}
            onChange={(e) => onNameChange(e.target.value)}
          />

          <div className='flex items-center gap-1 rounded-md border border-border p-0.5'>
            <ViewportButton
              active={viewport === 'iphone'}
              onClick={() => setViewport('iphone')}
              label='iPhone — 402 (base, below Tailwind sm)'
            >
              <Smartphone size={14} />
            </ViewportButton>
            <ViewportButton
              active={viewport === 'tablet'}
              onClick={() => setViewport('tablet')}
              label='Tablet — 768 (Tailwind md)'
            >
              <Tablet size={14} />
            </ViewportButton>
            <ViewportButton
              active={viewport === 'desktop'}
              onClick={() => setViewport('desktop')}
              label='Desktop — 1280 (Tailwind xl)'
            >
              <Monitor size={14} />
            </ViewportButton>
          </div>

          <Button onClick={onSave} disabled={isSaving || !name}>
            <Save size={16} />
            {isSaving ? 'Saving…' : isNew ? 'Create' : 'Save'}
          </Button>
        </header>

        <BuilderShell getIframeRect={getIframeRect} getIframeScale={getIframeScale}>
          <div className='border-b border-border bg-muted/30'>
            <SelectionBreadcrumb />
          </div>
          <div className='flex min-w-0 flex-1 overflow-hidden'>
            <div className='w-56 shrink-0 overflow-y-auto border-r border-border'>
              <LayoutComponentLibrary />
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
              {/* Outer box reserves the *scaled* visual footprint so layout
                  math matches what's drawn; the inner div shrinks the iframe
                  via `transform: scale` while keeping its device-size box. */}
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
                    responsiveCss={generateBreakpointCss(tree)}
                    onRectChange={(rect) => {
                      iframeRectRef.current = rect
                    }}
                  >
                    <Canvas maxWidth={viewportWidth} />
                  </CanvasFrame>
                </div>
              </div>
            </div>

            <ConditionalConfigSidebar />
          </div>
        </BuilderShell>
      </div>
    </BuilderProvider>
  )
}

/**
 * Right config rail — only mounted when a node is selected so the canvas
 * gets the full width whenever the user clicks empty space.
 */
function ConditionalConfigSidebar() {
  const selected = useSelectedNode()
  if (!selected) return null
  return (
    <div className='w-80 shrink-0 overflow-y-auto border-l border-border'>
      <ConfigPanel />
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
      className={`rounded px-2 py-1 text-xs transition-colors ${
        active ? 'bg-primary text-primary-foreground' : 'text-muted-foreground hover:bg-muted'
      }`}
      onClick={onClick}
    >
      {children}
    </button>
  )
}

/**
 * Bridges the BreakpointContext (set by the config panel's bp tabs) with the
 * layout builder's device viewport state. When the user clicks `md`, the
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
