import { useState, type ReactNode } from 'react'
import { ArrowLeft, Monitor, Save, Smartphone, Tablet } from 'lucide-react'
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
import { emailTemplatePresets, type EmailTemplatePreset } from '@/lib/builder-mjml'
import { PREVIEW_WIDTHS, type PreviewMode } from '@/lib/builder-mjml/types'
import { Pill } from '@/components/kit'
import { cn } from '@/lib/utils'

export interface PageEmailTemplateBuilderProps {
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
}: PageEmailTemplateBuilderProps) {
  const [viewport, setViewport] = useState<PreviewMode>('desktop')

  return (
    <BuilderProvider adapter={adapter} initialTree={tree} onChange={onTreeChange}>
      <div className='flex h-full flex-col'>
        <div className='flex items-center gap-3 border-b border-fk-line px-4 py-2'>
          <Button variant='ghost' size='sm' className='-ml-2 text-neutral-500' onClick={onBack}>
            <ArrowLeft className='size-3.5' />
            Emails
          </Button>

          <input
            type='text'
            className='min-w-0 flex-1 bg-transparent text-lg font-semibold tracking-tight outline-none placeholder:text-neutral-400'
            placeholder='Template name…'
            value={name}
            onChange={(event) => onNameChange(event.target.value)}
          />

          {isNew ? (
            <select
              aria-label='Email type'
              className='rounded-md border border-fk-line bg-white px-2 py-1 text-[13px]'
              value={emailType}
              onChange={(event) => onEmailTypeChange(event.target.value)}
            >
              {emailTypes.map((type) => (
                <option key={type.value} value={type.value}>
                  {type.label}
                </option>
              ))}
            </select>
          ) : (
            <Pill tone='neutral' mono>
              {emailType}
            </Pill>
          )}

          <div className='flex items-center gap-1 rounded-md border border-fk-line p-0.5'>
            <ViewportButton
              active={viewport === 'desktop'}
              label='Desktop'
              onClick={() => setViewport('desktop')}
            >
              <Monitor className='size-3.5' />
            </ViewportButton>
            <ViewportButton
              active={viewport === 'tablet'}
              label='Tablet'
              onClick={() => setViewport('tablet')}
            >
              <Tablet className='size-3.5' />
            </ViewportButton>
            <ViewportButton
              active={viewport === 'mobile'}
              label='Mobile'
              onClick={() => setViewport('mobile')}
            >
              <Smartphone className='size-3.5' />
            </ViewportButton>
          </div>

          <Button onClick={onSave} disabled={isSaving || !name}>
            <Save />
            {isSaving ? 'Saving…' : 'Save'}
          </Button>
        </div>

        <BuilderShell>
          <div className='flex flex-1 overflow-hidden'>
            <div className='w-56 shrink-0 overflow-y-auto border-r border-fk-line'>
              <PresetLibrary presets={emailTemplatePresets} onApplyPreset={onApplyPreset} />
              <ComponentLibrary />
            </div>

            <div
              className='flex-1 overflow-y-auto'
              style={{
                backgroundColor: 'var(--color-fk-line-soft)',
                backgroundImage:
                  'radial-gradient(circle, var(--color-fk-line) 1px, transparent 1px)',
                backgroundSize: '16px 16px',
              }}
            >
              <Canvas maxWidth={PREVIEW_WIDTHS[viewport]} />
            </div>

            <div className='w-80 shrink-0 overflow-y-auto border-l border-fk-line'>
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
  children: ReactNode
}) {
  return (
    <button
      type='button'
      title={label}
      aria-pressed={active}
      className={cn(
        'rounded px-2 py-1 text-xs transition-colors',
        active
          ? 'bg-fk-primary-soft font-medium text-fk-primary-text'
          : 'text-neutral-500 hover:bg-neutral-50'
      )}
      onClick={onClick}
    >
      {children}
    </button>
  )
}
