import type { ReactNode } from 'react'
import type { BuilderNode } from '../../builder-core'

interface Props {
  node: BuilderNode
  isSelected: boolean
  children?: ReactNode
}

export function ContainerBlock({ node, isSelected, children }: Props) {
  const direction = (node.props.direction as string) || 'column'
  const align = (node.props.align as string) || 'stretch'
  const gap = (node.props.gap as string) || '12px'
  const padding = (node.props.padding as string) || '16px'
  const backgroundColor = (node.props.backgroundColor as string) || undefined
  const borderRadius = (node.props.borderRadius as string) || undefined
  const width = (node.props.width as string) || '100%'

  return (
    <div
      className={`group/container relative transition-all ${
        isSelected ? 'ring-2 ring-primary' : 'hover:ring-1 hover:ring-dashed hover:ring-border'
      }`}
      style={{
        display: 'flex',
        flexDirection: direction as 'row' | 'column',
        alignItems: align,
        gap,
        padding,
        backgroundColor,
        borderRadius,
        width,
        maxWidth: '100%',
      }}
    >
      <span className='absolute left-1 top-1 z-10 rounded bg-muted/80 px-1 py-0.5 text-[10px] font-medium text-muted-foreground opacity-0 transition-opacity group-hover/container:opacity-100'>
        Container
      </span>
      {children}
    </div>
  )
}
