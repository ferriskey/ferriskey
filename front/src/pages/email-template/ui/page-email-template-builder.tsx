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
import { MjmlPreview } from '@/lib/builder-mjml'
import { ArrowLeft, Eye, Layers, Save } from 'lucide-react'
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
}

type Tab = 'builder' | 'preview'

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
}: Props) {
  const [tab, setTab] = useState<Tab>('builder')

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
            <button
              type='button'
              className={`rounded px-2 py-1 text-xs transition-colors ${
                tab === 'builder' ? 'bg-primary text-primary-foreground' : 'text-muted-foreground hover:bg-muted'
              }`}
              onClick={() => setTab('builder')}
            >
              <Layers size={14} />
            </button>
            <button
              type='button'
              className={`rounded px-2 py-1 text-xs transition-colors ${
                tab === 'preview' ? 'bg-primary text-primary-foreground' : 'text-muted-foreground hover:bg-muted'
              }`}
              onClick={() => setTab('preview')}
            >
              <Eye size={14} />
            </button>
          </div>

          <Button onClick={onSave} disabled={isSaving || !name}>
            <Save size={16} />
            {isSaving ? 'Saving...' : 'Save'}
          </Button>
        </div>

        {/* Content */}
        {tab === 'builder' ? (
          <BuilderShell>
            <div className='flex flex-1 overflow-hidden'>
              {/* Component Library */}
              <div className='w-56 shrink-0 overflow-y-auto border-r border-border'>
                <ComponentLibrary />
              </div>

              {/* Canvas */}
              <div className='flex-1 overflow-y-auto p-4'>
                <Canvas />
              </div>

              {/* Config Panel */}
              <div className='w-72 shrink-0 overflow-y-auto border-l border-border'>
                <ConfigPanel />
              </div>
            </div>
          </BuilderShell>
        ) : (
          <div className='flex-1 overflow-y-auto p-4'>
            <MjmlPreview />
          </div>
        )}
      </div>
    </BuilderProvider>
  )
}
