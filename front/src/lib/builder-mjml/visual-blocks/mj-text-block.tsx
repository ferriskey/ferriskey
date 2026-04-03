import { useMemo } from 'react'
import type { BuilderNode } from '../../builder-core'

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

  return (
    <div
      className={`transition-all ${
        isSelected ? 'ring-2 ring-primary' : 'hover:ring-1 hover:ring-dashed hover:ring-border'
      }`}
      style={{
        fontSize,
        color,
        textAlign: align as 'left' | 'center' | 'right',
        padding,
        fontWeight,
        fontFamily,
        lineHeight,
      }}
      dangerouslySetInnerHTML={{ __html: sanitizedContent }}
    />
  )
}
