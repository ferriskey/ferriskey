import type { ReactNode } from 'react'
import type { BuilderNode } from '../../builder-core'

interface Props {
  node: BuilderNode
  isSelected: boolean
  children?: ReactNode
}

export function MjSectionBlock({ node, isSelected, children }: Props) {
  const bgColor = (node.props['background-color'] as string) || undefined
  const padding = (node.props['padding'] as string) || '10px'

  return (
    <div
      className={`group/section relative w-full transition-all ${
        isSelected ? 'ring-2 ring-primary' : 'hover:ring-1 hover:ring-dashed hover:ring-border'
      }`}
      style={{ backgroundColor: bgColor, padding }}
    >
      <span className='absolute left-1 top-1 z-10 rounded bg-muted/80 px-1 py-0.5 text-[10px] font-medium text-muted-foreground opacity-0 transition-opacity group-hover/section:opacity-100'>
        Section
      </span>
      <div className='flex flex-row gap-0'>
        {children}
      </div>
    </div>
  )
}
