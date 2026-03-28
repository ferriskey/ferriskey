import { useEffect, useRef, useState, useMemo, useCallback } from 'react'
import { Monitor, Tablet, Smartphone } from 'lucide-react'
import { useBuilder } from '../builder-core'
import { PREVIEW_WIDTHS, type PreviewMode } from './types'
import { treeToHtml } from './renderer'

export function MjmlPreview() {
  const { tree } = useBuilder()
  const [mode, setMode] = useState<PreviewMode>('desktop')
  const iframeRef = useRef<HTMLIFrameElement>(null)

  const html = useMemo(() => {
    if (tree.length === 0) return ''
    return treeToHtml(tree)
  }, [tree])

  const writeToIframe = useCallback(() => {
    const iframe = iframeRef.current
    if (!iframe) return
    const doc = iframe.contentDocument
    if (!doc) return
    doc.open()
    doc.write(html)
    doc.close()
  }, [html])

  useEffect(() => {
    writeToIframe()
  }, [writeToIframe])

  const width = PREVIEW_WIDTHS[mode]

  return (
    <div className='flex flex-col gap-2'>
      <div className='flex items-center justify-center gap-1'>
        <PreviewModeButton
          active={mode === 'desktop'}
          onClick={() => setMode('desktop')}
          label='Desktop'
        >
          <Monitor size={14} />
        </PreviewModeButton>
        <PreviewModeButton
          active={mode === 'tablet'}
          onClick={() => setMode('tablet')}
          label='Tablet'
        >
          <Tablet size={14} />
        </PreviewModeButton>
        <PreviewModeButton
          active={mode === 'mobile'}
          onClick={() => setMode('mobile')}
          label='Mobile'
        >
          <Smartphone size={14} />
        </PreviewModeButton>
      </div>

      <div className='flex justify-center overflow-auto rounded-lg border border-border bg-muted/30 p-4'>
        {html ? (
          <iframe
            ref={iframeRef}
            title='Email Preview'
            style={{ width, height: 600, border: 'none' }}
            className='rounded bg-white transition-all duration-200'
            sandbox='allow-same-origin'
          />
        ) : (
          <div className='flex h-[600px] items-center justify-center text-sm text-muted-foreground' style={{ width }}>
            Build your email to see a preview
          </div>
        )}
      </div>
    </div>
  )
}

function PreviewModeButton({
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
      className={`rounded p-1.5 transition-colors ${
        active
          ? 'bg-primary/10 text-primary'
          : 'text-muted-foreground hover:bg-muted'
      }`}
      onClick={onClick}
    >
      {children}
    </button>
  )
}
