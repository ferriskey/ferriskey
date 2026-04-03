import type { BuilderNode } from '../../builder-core'
import { useBuilderContext } from '../../builder-core'
import { InlineTextEditor } from './inline-text-editor'

interface Props {
  node: BuilderNode
  isSelected: boolean
}

export function MjButtonBlock({ node, isSelected }: Props) {
  const { updateNode, adapter } = useBuilderContext()

  const bgColor = (node.props['background-color'] as string) || '#007bff'
  const color = (node.props['color'] as string) || '#ffffff'
  const fontSize = (node.props['font-size'] as string) || '14px'
  const borderRadius = (node.props['border-radius'] as string) || '4px'
  const align = (node.props['align'] as string) || 'center'
  const padding = (node.props['padding'] as string) || '10px 25px'

  return (
    <div
      className={`transition-all ${
        isSelected ? '' : 'hover:ring-1 hover:ring-dashed hover:ring-border'
      }`}
      style={{ padding, textAlign: align as 'left' | 'center' | 'right' }}
    >
      <div
        className='relative'
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
        {isSelected ? (
          <InlineTextEditor
            content={node.content || 'Click me'}
            onChange={(html) => updateNode(node.id, { content: html })}
            variables={adapter.variables}
          />
        ) : (
          node.content || 'Click me'
        )}
      </div>
    </div>
  )
}
