import type { BuilderNode } from '../../builder-core'

interface Props {
  node: BuilderNode
  isSelected: boolean
}

export function MjButtonBlock({ node, isSelected }: Props) {
  const bgColor = (node.props['background-color'] as string) || '#007bff'
  const color = (node.props['color'] as string) || '#ffffff'
  const fontSize = (node.props['font-size'] as string) || '14px'
  const borderRadius = (node.props['border-radius'] as string) || '4px'
  const align = (node.props['align'] as string) || 'center'
  const padding = (node.props['padding'] as string) || '10px 25px'

  return (
    <div
      className={`transition-all ${
        isSelected ? 'ring-2 ring-primary' : 'hover:ring-1 hover:ring-dashed hover:ring-border'
      }`}
      style={{ padding, textAlign: align as 'left' | 'center' | 'right' }}
    >
      <div
        style={{
          display: 'inline-block',
          backgroundColor: bgColor,
          color,
          fontSize,
          borderRadius,
          padding: '10px 25px',
          cursor: 'default',
          fontWeight: 500,
        }}
      >
        {node.content || 'Click me'}
      </div>
    </div>
  )
}
