import { ImageIcon } from 'lucide-react'
import type { BuilderNode } from '../../builder-core'

interface Props {
  node: BuilderNode
  isSelected: boolean
}

export function MjImageBlock({ node, isSelected }: Props) {
  const src = node.props['src'] as string | undefined
  const alt = (node.props['alt'] as string) || ''
  const width = (node.props['width'] as string) || '100%'
  const align = (node.props['align'] as string) || 'center'
  const padding = (node.props['padding'] as string) || '10px 25px'

  return (
    <div
      className={`transition-all ${
        isSelected ? 'ring-2 ring-primary' : 'hover:ring-1 hover:ring-dashed hover:ring-border'
      }`}
      style={{ padding, textAlign: align as 'left' | 'center' | 'right' }}
    >
      {src ? (
        <img
          src={src}
          alt={alt}
          style={{ width, maxWidth: '100%', height: 'auto', display: 'inline-block' }}
        />
      ) : (
        <div className='flex h-[120px] items-center justify-center rounded bg-muted/50 text-muted-foreground'>
          <ImageIcon size={32} />
        </div>
      )}
    </div>
  )
}
