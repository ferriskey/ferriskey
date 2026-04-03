import { useMemo } from 'react'
import type { BuilderNode } from '../../builder-core'
import { useBuilderContext } from '../../builder-core'
import { InlineTextEditor } from './inline-text-editor'

interface Props {
  node: BuilderNode
  isSelected: boolean
}

const STYLE_PROPS = ['font-size', 'color', 'text-align', 'font-weight', 'font-family', 'line-height'] as const

function stripConflictingInlineStyles(html: string): string {
  const pattern = new RegExp(`\\b(${STYLE_PROPS.join('|')})\\s*:[^;]*;?`, 'gi')
  return html.replace(/style="([^"]*)"/gi, (_match, styles: string) => {
    const cleaned = styles.replace(pattern, '').replace(/\s{2,}/g, ' ').trim()
    return cleaned ? `style="${cleaned}"` : ''
  })
}

export function MjTextBlock({ node, isSelected }: Props) {
  const { updateNode, adapter } = useBuilderContext()

  const fontSize = (node.props['font-size'] as string) || '14px'
  const color = (node.props['color'] as string) || '#333333'
  const align = (node.props['align'] as string) || 'left'
  const padding = (node.props['padding'] as string) || '10px 25px'
  const fontWeight = (node.props['font-weight'] as string) || undefined
  const fontFamily = (node.props['font-family'] as string) || undefined
  const lineHeight = (node.props['line-height'] as string) || undefined

  const sanitizedContent = useMemo(
    () => stripConflictingInlineStyles(node.content || '<p>Text</p>'),
    [node.content],
  )

  const style = {
    fontSize,
    color,
    textAlign: align as 'left' | 'center' | 'right',
    padding,
    fontWeight,
    fontFamily,
    lineHeight,
  }

  if (isSelected) {
    return (
      <div
        className='relative transition-all'
        style={style}
        onPointerDown={(e) => e.stopPropagation()}
        onClick={(e) => e.stopPropagation()}
      >
        <InlineTextEditor
          content={sanitizedContent}
          onChange={(html) => updateNode(node.id, { content: html })}
          variables={adapter.variables}
        />
      </div>
    )
  }

  return (
    <div
      className='relative transition-all hover:ring-1 hover:ring-dashed hover:ring-border'
      style={style}
      dangerouslySetInnerHTML={{ __html: sanitizedContent }}
    />
  )
}
