import type { EmailTemplatePreset } from '@/lib/builder-mjml/presets'
import { FileText } from 'lucide-react'
import { useState } from 'react'
import { useBuilder } from '../hooks'
import { regenerateIds } from '../utils'

interface PresetLibraryProps {
  presets: EmailTemplatePreset[]
  onApplyPreset: (preset: EmailTemplatePreset) => void
}

export function PresetLibrary({ presets, onApplyPreset }: PresetLibraryProps) {
  const { tree, setTree } = useBuilder()
  const [pendingPreset, setPendingPreset] = useState<EmailTemplatePreset | null>(null)

  function applyPreset(preset: EmailTemplatePreset) {
    setTree(regenerateIds(preset.tree))
    onApplyPreset(preset)
    setPendingPreset(null)
  }

  function handleClick(preset: EmailTemplatePreset) {
    if (tree.length > 0) {
      setPendingPreset(preset)
    } else {
      applyPreset(preset)
    }
  }

  return (
    <div className='flex flex-col gap-1.5 p-2'>
      <h3 className='px-1 text-xs font-medium uppercase tracking-wider text-muted-foreground'>
        Presets
      </h3>
      <div className='flex flex-col gap-1'>
        {presets.map((preset) => (
          <button
            key={preset.id}
            type='button'
            onClick={() => handleClick(preset)}
            className='flex items-start gap-2 rounded-md border border-border bg-card p-2 text-left text-sm transition-colors hover:bg-accent'
          >
            <span className='mt-0.5 flex h-5 w-5 shrink-0 items-center justify-center text-muted-foreground'>
              <FileText size={14} />
            </span>
            <div className='min-w-0 flex-1'>
              <div className='flex items-center gap-1.5'>
                <span className='truncate font-medium'>{preset.name}</span>
              </div>
              <p className='mt-0.5 truncate text-xs text-muted-foreground'>{preset.description}</p>
            </div>
          </button>
        ))}
      </div>

      {pendingPreset && (
        <div className='rounded-md border border-border bg-card p-2'>
          <p className='mb-2 text-xs text-muted-foreground'>
            This will replace your current layout. Continue?
          </p>
          <div className='flex gap-1'>
            <button
              type='button'
              onClick={() => applyPreset(pendingPreset)}
              className='flex-1 rounded bg-primary px-2 py-1 text-xs text-primary-foreground transition-colors hover:bg-primary/90'
            >
              Replace
            </button>
            <button
              type='button'
              onClick={() => setPendingPreset(null)}
              className='flex-1 rounded border border-border px-2 py-1 text-xs transition-colors hover:bg-muted'
            >
              Cancel
            </button>
          </div>
        </div>
      )}
    </div>
  )
}
