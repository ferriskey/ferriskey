import { Button } from '@/components/ui/button'
import {
  BuilderProvider,
  BuilderShell,
  Canvas,
  ComponentLibrary,
  ConfigPanel,
  PresetLibrary,
  type BuilderAdapter,
  type BuilderNode,
} from '@/lib/builder-core'
import { type EmailTemplatePreset, emailTemplatePresets } from '@/lib/builder-mjml'
import { PREVIEW_WIDTHS, type PreviewMode } from '@/lib/builder-mjml/types'
import { ArrowLeft, Monitor, Smartphone, Save, Tablet } from 'lucide-react'
import { useState } from 'react'

interface Props {
  adapter: BuilderAdapter
  tree: BuilderNode[]
  onTreeChange: (tree: BuilderNode[]) => void
  name: string
  onNameChange: (name: string) => void
  emailType: string
  onEmailTypeChange: (type: string) => void
  emailTypes: { label: string; value: string }[]
  isNew: boolean
  isSaving: boolean
  onSave: () => void
  onBack: () => void
  onApplyPreset: (preset: EmailTemplatePreset) => void
}

export default function PageEmailTemplateBuilder({
  adapter,
  tree,
  onTreeChange,
  name,
  onNameChange,
  emailType,
  onEmailTypeChange,
  emailTypes,
  isNew,
  isSaving,
  onSave,
  onBack,
  onApplyPreset,
}: Props) {
  const [viewport, setViewport] = useState<PreviewMode>('desktop')
  const viewportWidth = PREVIEW_WIDTHS[viewport]

  return (
    <BuilderProvider adapter={adapter} initialTree={tree} onChange={onTreeChange}>
      <div className='flex h-full flex-col'>
        {/* Header */}
        <div className='flex items-center gap-3 border-b border-border px-4 py-2'>
          <Button variant='ghost' size='icon' onClick={onBack}>
            <ArrowLeft size={16} />
          </Button>

          <input
            type='text'
            className='flex-1 bg-transparent text-lg font-medium outline-none placeholder:text-muted-foreground'
            placeholder='Template name...'
            value={name}
            onChange={(e) => onNameChange(e.target.value)}
          />

          {isNew && (
            <select
              className='rounded border border-border bg-background px-2 py-1 text-sm'
              value={emailType}
              onChange={(e) => onEmailTypeChange(e.target.value)}
            >
              {emailTypes.map((t) => (
                <option key={t.value} value={t.value}>
                  {t.label}
                </option>
              ))}
            </select>
          )}

          <div className='flex items-center gap-1 rounded-md border border-border p-0.5'>
            <ViewportButton active={viewport === 'desktop'} onClick={() => setViewport('desktop')} label='Desktop'>
              <Monitor size={14} />
            </ViewportButton>
            <ViewportButton active={viewport === 'tablet'} onClick={() => setViewport('tablet')} label='Tablet'>
              <Tablet size={14} />
            </ViewportButton>
            <ViewportButton active={viewport === 'mobile'} onClick={() => setViewport('mobile')} label='Mobile'>
              <Smartphone size={14} />
            </ViewportButton>
          </div>

          <Button onClick={onSave} disabled={isSaving || !name}>
            <Save size={16} />
            {isSaving ? 'Saving...' : 'Save'}
          </Button>
        </div>

        {/* Content */}
        <BuilderShell>
          <div className='flex flex-1 overflow-hidden'>
            {/* Component Library */}
            <div className='w-56 shrink-0 overflow-y-auto border-r border-border'>
              <PresetLibrary presets={emailTemplatePresets} onApplyPreset={onApplyPreset} />
              <ComponentLibrary />
            </div>

            {/* Visual Canvas */}
            <div className='flex-1 overflow-y-auto'>
              <Canvas maxWidth={viewportWidth} />
            </div>

            {/* Right Panel: Config */}
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
