import type { BuilderNode } from '../../builder-core'

interface Props {
  node: BuilderNode
  isSelected: boolean
}

export function MjDividerBlock({ node, isSelected }: Props) {
  const borderColor = (node.props['border-color'] as string) || '#cccccc'
  const borderStyle = (node.props['border-style'] as string) || 'solid'
  const borderWidth = (node.props['border-width'] as string) || '1px'
  const padding = (node.props['padding'] as string) || '10px 25px'

  return (
    <div
      className={`transition-all ${
        isSelected ? 'ring-2 ring-primary' : 'hover:ring-1 hover:ring-dashed hover:ring-border'
      }`}
      style={{ padding }}
    >
      <hr
        style={{
          border: 'none',
          borderTop: `${borderWidth} ${borderStyle} ${borderColor}`,
          margin: 0,
        }}
      />
    </div>
  )
}
