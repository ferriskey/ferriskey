import type { ReactNode } from 'react'
import type { BuilderNode, ComponentDefinition } from '../../builder-core'

interface Props {
  node: BuilderNode
  isSelected: boolean
  componentDef?: ComponentDefinition
  children?: ReactNode
}

export function MjFallbackBlock({ node, isSelected, componentDef, children }: Props) {
  return (
    <div
      className={`flex min-h-[40px] flex-col items-center justify-center gap-1 rounded border border-dashed border-border/60 bg-muted/20 p-3 transition-all ${
        isSelected ? 'ring-2 ring-primary' : 'hover:ring-1 hover:ring-dashed hover:ring-border'
      }`}
    >
      <div className='flex items-center gap-1.5 text-xs text-muted-foreground'>
        {componentDef?.icon && (
          <span className='flex h-4 w-4 items-center justify-center'>{componentDef.icon}</span>
        )}
        <span className='font-medium'>{componentDef?.label ?? node.type}</span>
      </div>
      {children}
    </div>
  )
}
