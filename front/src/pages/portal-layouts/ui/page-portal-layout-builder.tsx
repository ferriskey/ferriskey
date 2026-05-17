import { Button } from '@/components/ui/button'
import {
  BuilderProvider,
  BuilderShell,
  Canvas,
  ComponentLibrary,
  ConfigPanel,
  type BuilderAdapter,
  type BuilderNode,
} from '@/lib/builder-core'
import { ArrowLeft, Monitor, Save, Smartphone, Tablet } from 'lucide-react'
import { useState, type CSSProperties } from 'react'

type Viewport = 'desktop' | 'tablet' | 'mobile'

const VIEWPORT_WIDTHS: Record<Viewport, number> = {
  desktop: 1024,
  tablet: 768,
  mobile: 375,
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

        <BuilderShell>
          <div className='flex min-w-0 flex-1 overflow-hidden'>
            <div className='w-56 shrink-0 overflow-y-auto border-r border-border'>
              <ComponentLibrary />
            </div>

            <div
              className='flex min-w-0 flex-1 justify-center overflow-auto bg-muted/30 p-6'
              style={cssVars}
            >
              <div
                className='shrink-0 self-start rounded-lg border border-border bg-background shadow-sm transition-all duration-200'
                style={{ width: VIEWPORT_WIDTHS[viewport] }}
              >
                <Canvas maxWidth={VIEWPORT_WIDTHS[viewport]} />
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
