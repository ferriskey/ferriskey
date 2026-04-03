import { useEffect, useRef, useMemo, useCallback } from 'react'
import { useBuilder } from '../builder-core'
import { treeToHtml } from './renderer'

interface MjmlPreviewProps {
  previewWidth?: number
}

export function MjmlPreview({ previewWidth = 600 }: MjmlPreviewProps) {
  const { tree } = useBuilder()
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

  const width = previewWidth

  return (
    <div className='flex flex-col gap-2'>
      <div className='flex justify-center overflow-auto rounded-lg border border-border bg-muted/30 p-2'>
        {html ? (
          <iframe
            ref={iframeRef}
            title='Email Preview'
            style={{ width, height: 400, border: 'none' }}
            className='rounded bg-white transition-all duration-200'
            sandbox='allow-same-origin'
          />
        ) : (
          <div className='flex h-[400px] items-center justify-center text-sm text-muted-foreground' style={{ width }}>
            Build your email to see a preview
          </div>
        )}
      </div>
    </div>
  )
}
