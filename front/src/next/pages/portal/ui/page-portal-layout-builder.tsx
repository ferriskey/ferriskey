import { useCallback, useEffect, useRef, useState, type CSSProperties } from 'react'
import { ArrowLeft, Monitor, Save, Smartphone, Tablet } from 'lucide-react'
import { Button } from '@/components/kit/button'
import { Input } from '@/components/ui/input'
import { Pill } from '@/components/kit'
import { cn } from '@/lib/utils'
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
import { generateBreakpointCss, useIframeFit } from '@/lib/builder-portal'
import { CanvasFrame } from '@/lib/builder-portal/components/canvas-frame'
import { LayoutComponentLibrary } from '@/pages/portal-layouts/components/layout-component-library'

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

const BREAKPOINT_TO_VIEWPORT: Record<Breakpoint, Viewport> = {
  sm: 'tablet',
  md: 'tablet',
  lg: 'desktop',
  xl: 'desktop',
}

export interface PagePortalLayoutBuilderProps {
  adapter: BuilderAdapter
  tree: BuilderNode[]
  name: string
  isNew: boolean
  isSaving: boolean
  isDefault: boolean
  usedBy: string[]
  cssVars: CSSProperties
  onTreeChange: (tree: BuilderNode[]) => void
  onNameChange: (name: string) => void
  onSave: () => void
  onBack: () => void
}

export default function PagePortalLayoutBuilder({
  adapter,
  tree,
  name,
  isNew,
  isSaving,
  isDefault,
  usedBy,
  cssVars,
  onTreeChange,
  onNameChange,
  onSave,
  onBack,
}: PagePortalLayoutBuilderProps) {
  const [viewport, setViewport] = useState<Viewport>('desktop')
  const iframeRectRef = useRef<DOMRect | null>(null)
  const iframeScaleRef = useRef<number>(1)
  const getIframeRect = useCallback(() => iframeRectRef.current, [])
  const getIframeScale = useCallback(() => iframeScaleRef.current, [])
  const handleBreakpointChange = useCallback((bp: Breakpoint | null) => {
    if (bp) setViewport(BREAKPOINT_TO_VIEWPORT[bp])
  }, [])

  const viewportWidth = VIEWPORT_WIDTHS[viewport]
  const viewportHeight = VIEWPORT_HEIGHTS[viewport]
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
      <div className='flex h-full min-h-0 w-full min-w-0 flex-col overflow-hidden'>
        <header className='flex shrink-0 flex-wrap items-center gap-3 border-b border-fk-line px-5 py-2.5'>
          <Button variant='ghost' size='sm' className='-ml-2 text-neutral-500 dark:text-neutral-400' onClick={onBack}>
            <ArrowLeft className='size-3.5' />
            Portal
          </Button>

          <Input
            value={name}
            onChange={(e) => onNameChange(e.target.value)}
            placeholder='Layout name'
            aria-label='Layout name'
            className='max-w-xs'
          />

          {isDefault && <Pill tone='violet'>default</Pill>}
          {usedBy.length > 0 && (
            <Pill tone='info'>
              {usedBy.length} theme{usedBy.length > 1 ? 's' : ''}
            </Pill>
          )}

          <div className='ml-auto flex items-center gap-3'>
            <div className='flex items-center gap-1 rounded-md border border-fk-line p-0.5'>
              <ViewportButton
                active={viewport === 'iphone'}
                onClick={() => setViewport('iphone')}
                label='iPhone — 402 (base, below Tailwind sm)'
              >
                <Smartphone className='size-3.5' />
              </ViewportButton>
              <ViewportButton
                active={viewport === 'tablet'}
                onClick={() => setViewport('tablet')}
                label='Tablet — 768 (Tailwind md)'
              >
                <Tablet className='size-3.5' />
              </ViewportButton>
              <ViewportButton
                active={viewport === 'desktop'}
                onClick={() => setViewport('desktop')}
                label='Desktop — 1280 (Tailwind xl)'
              >
                <Monitor className='size-3.5' />
              </ViewportButton>
            </div>

            <Button onClick={onSave} disabled={isSaving || !name}>
              <Save className='size-4' />
              {isSaving ? 'Saving…' : isNew ? 'Create' : 'Save'}
            </Button>
          </div>
        </header>

        <BuilderShell getIframeRect={getIframeRect} getIframeScale={getIframeScale}>
          <div className='shrink-0 border-b border-fk-line bg-neutral-50 dark:bg-neutral-900'>
            <SelectionBreadcrumb />
          </div>
          <div className='flex min-h-0 min-w-0 flex-1 overflow-hidden'>
            <div className='w-56 shrink-0 overflow-y-auto border-r border-fk-line'>
              <LayoutComponentLibrary />
            </div>

            <div
              ref={canvasAreaRef}
              className='flex min-w-0 flex-1 items-start justify-center overflow-hidden p-5'
              style={{
                backgroundColor: '#f8f9fa',
                backgroundImage: 'radial-gradient(circle, #d1d5db 1px, transparent 1px)',
                backgroundSize: '20px 20px',
              }}
            >
              <div
                className='shrink-0 self-start overflow-hidden rounded-sm border border-fk-line bg-white dark:bg-neutral-900 shadow-sm transition-all duration-200'
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
                    <Canvas maxWidth={viewportWidth} padded={false} />
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

function ConditionalConfigSidebar() {
  const selected = useSelectedNode()
  if (!selected) return null
  return (
    <div className='w-80 shrink-0 overflow-y-auto border-l border-fk-line'>
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
      onClick={onClick}
      className={cn(
        'cursor-pointer rounded px-2 py-1 text-xs transition-colors',
        active ? 'bg-fk-primary-soft text-fk-primary-text' : 'text-neutral-500 hover:bg-neutral-100 dark:text-neutral-400 dark:hover:bg-neutral-800'
      )}
    >
      {children}
    </button>
  )
}

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
