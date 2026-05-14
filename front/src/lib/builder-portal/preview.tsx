import type { CSSProperties, ReactNode } from 'react'
import { useBuilder } from '../builder-core'
import { treeToReactNode } from './renderer'

interface PortalPreviewProps {
  /** Optional CSS variable map (typically derived from PortalThemeConfig). */
  cssVars?: CSSProperties
  /** Slot content for <page-content /> nodes (used when previewing a layout). */
  pageContent?: ReactNode
  /** Tailwind class for the preview viewport. */
  className?: string
}

export function PortalPreview({ cssVars, pageContent, className }: PortalPreviewProps) {
  const { tree } = useBuilder()

  return (
    <div className={className ?? 'flex h-full w-full justify-center overflow-auto p-4'}>
      <div
        style={{
          ...cssVars,
          backgroundColor: 'var(--fk-color-page-bg, #f3f4f6)',
          width: '100%',
          maxWidth: 480,
          borderRadius: 'var(--fk-radius-widget, 8px)',
          boxShadow: 'var(--fk-shadow-widget, 0 1px 2px rgb(0 0 0 / 0.05))',
        }}
      >
        <div
          style={{
            backgroundColor: 'var(--fk-color-widget-bg, #ffffff)',
            borderRadius: 'var(--fk-radius-widget, 8px)',
            overflow: 'hidden',
          }}
        >
          {tree.length > 0 ? (
            treeToReactNode(tree, { pageContent })
          ) : (
            <div className='flex h-[400px] items-center justify-center text-sm text-muted-foreground'>
              Drag blocks to start building
            </div>
          )}
        </div>
      </div>
    </div>
  )
}
