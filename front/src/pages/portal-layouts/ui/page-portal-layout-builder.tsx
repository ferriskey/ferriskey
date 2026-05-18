import { Button } from '@/components/ui/button'
import {
  BuilderProvider,
  BuilderShell,
  Canvas,
  ConfigPanel,
  type BuilderAdapter,
  type BuilderNode,
} from '@/lib/builder-core'
import { CanvasFrame } from '@/lib/builder-portal/components/canvas-frame'
import { LayoutComponentLibrary } from '../components/layout-component-library'
import { ArrowLeft, Monitor, Save, Smartphone, Tablet } from 'lucide-react'
import { useCallback, useRef, useState, type CSSProperties } from 'react'

type Viewport = 'desktop' | 'tablet' | 'mobile'

// Desktop fills the editor surface and relies on the outer p-5 padding to
// reveal a strip of dots on each side. Tablet/mobile use fixed device widths
// centered inside the dotted area.
const VIEWPORT_WIDTHS: Record<Viewport, number | string> = {
  desktop: '100%',
  tablet: 768,
  mobile: 375,
}

// Heights drive what `100vh` resolves to inside the iframe — they let the
// canvas mount target fill a device-shaped viewport.
const VIEWPORT_HEIGHTS: Record<Viewport, number> = {
  desktop: 800,
  tablet: 1024,
  mobile: 812,
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
  const getIframeRect = useCallback(() => iframeRectRef.current, [])

  return (
    <BuilderProvider adapter={adapter} initialTree={tree} onChange={onTreeChange}>
      <div className='flex h-[calc(100vh-3rem)] w-full min-w-0 flex-col overflow-hidden'>
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
              active={viewport === 'desktop'}
              onClick={() => setViewport('desktop')}
              label='Desktop'
            >
              <Monitor size={14} />
            </ViewportButton>
            <ViewportButton
              active={viewport === 'tablet'}
              onClick={() => setViewport('tablet')}
              label='Tablet'
            >
              <Tablet size={14} />
            </ViewportButton>
            <ViewportButton
              active={viewport === 'mobile'}
              onClick={() => setViewport('mobile')}
              label='Mobile'
            >
              <Smartphone size={14} />
            </ViewportButton>
          </div>

          <Button onClick={onSave} disabled={isSaving || !name}>
            <Save size={16} />
            {isSaving ? 'Saving…' : isNew ? 'Create' : 'Save'}
          </Button>
        </header>

        <BuilderShell getIframeRect={getIframeRect}>
          <div className='flex min-w-0 flex-1 overflow-hidden'>
            <div className='w-56 shrink-0 overflow-y-auto border-r border-border'>
              <LayoutComponentLibrary />
            </div>

            <div
              className='flex min-w-0 flex-1 justify-center overflow-auto p-5'
              style={{
                backgroundColor: '#f8f9fa',
                backgroundImage:
                  'radial-gradient(circle, #d1d5db 1px, transparent 1px)',
                backgroundSize: '20px 20px',
              }}
            >
              <div
                className='self-start overflow-hidden rounded-lg border border-border bg-background shadow-sm transition-all duration-200'
                // Desktop fills the available width so the iframe can resolve
                // its own `width: 100%` against a concrete pixel value;
                // tablet/mobile take the iframe's fixed width via `w-auto`.
                style={{
                  width: viewport === 'desktop' ? '100%' : 'auto',
                  flexShrink: 0,
                }}
              >
                <CanvasFrame
                  width={VIEWPORT_WIDTHS[viewport]}
                  height={VIEWPORT_HEIGHTS[viewport]}
                  cssVars={cssVars}
                  onRectChange={(rect) => {
                    iframeRectRef.current = rect
                  }}
                >
                  <Canvas
                    maxWidth={
                      typeof VIEWPORT_WIDTHS[viewport] === 'number'
                        ? (VIEWPORT_WIDTHS[viewport] as number)
                        : 1600
                    }
                  />
                </CanvasFrame>
              </div>
            </div>

            <div className='w-80 shrink-0 overflow-y-auto border-l border-border'>
              <ConfigPanel />
            </div>
          </div>
        </BuilderShell>
      </div>
    </BuilderProvider>
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
