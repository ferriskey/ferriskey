import type { BuilderNode } from '../../builder-core'

interface Props {
  node: BuilderNode
  isSelected: boolean
}

export function MjSpacerBlock({ node, isSelected }: Props) {
  const height = (node.props['height'] as string) || '20px'

  return (
    <div
      className={`relative transition-all ${
        isSelected ? 'ring-2 ring-primary' : 'hover:ring-1 hover:ring-dashed hover:ring-border'
      }`}
      style={{ height }}
    >
      <span className='absolute inset-0 flex items-center justify-center text-[10px] text-muted-foreground/50'>
        Spacer
      </span>
    </div>
  )
}
