import type { ReactNode } from 'react'
import type { BuilderNode } from '../../builder-core'

interface Props {
  node: BuilderNode
  isSelected: boolean
  children?: ReactNode
}

export function MjColumnBlock({ node, isSelected, children }: Props) {
  const bgColor = (node.props['background-color'] as string) || undefined
  const width = (node.props['width'] as string) || undefined
  const hasChildren = node.children.length > 0

  return (
    <div
      className={`relative flex-1 transition-all ${
        isSelected ? 'ring-2 ring-primary' : 'hover:ring-1 hover:ring-dotted hover:ring-border'
      }`}
      style={{ backgroundColor: bgColor, width }}
    >
      <div className='flex min-h-[40px] flex-col'>
        {hasChildren ? (
          children
        ) : (
          <div className='flex min-h-[60px] items-center justify-center border border-dashed border-border/50 text-xs text-muted-foreground'>
            Drop here
          </div>
        )}
      </div>
    </div>
  )
}
