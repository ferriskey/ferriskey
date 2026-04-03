import type { ReactNode } from 'react'
import type { BuilderNode, ComponentDefinition } from '../../builder-core'
import { MjSectionBlock } from './mj-section-block'
import { MjColumnBlock } from './mj-column-block'
import { MjTextBlock } from './mj-text-block'
import { MjImageBlock } from './mj-image-block'
import { MjButtonBlock } from './mj-button-block'
import { MjDividerBlock } from './mj-divider-block'
import { MjSpacerBlock } from './mj-spacer-block'
import { MjFallbackBlock } from './mj-fallback-block'

export function renderVisualBlock(
  node: BuilderNode,
  isSelected: boolean,
  children: ReactNode | undefined,
  componentDef?: ComponentDefinition,
): ReactNode {
  switch (node.type) {
    case 'mj-section':
      return (
        <MjSectionBlock node={node} isSelected={isSelected}>
          {children}
        </MjSectionBlock>
      )
    case 'mj-column':
      return (
        <MjColumnBlock node={node} isSelected={isSelected}>
          {children}
        </MjColumnBlock>
      )
    case 'mj-text':
      return <MjTextBlock node={node} isSelected={isSelected} />
    case 'mj-image':
      return <MjImageBlock node={node} isSelected={isSelected} />
    case 'mj-button':
      return <MjButtonBlock node={node} isSelected={isSelected} />
    case 'mj-divider':
      return <MjDividerBlock node={node} isSelected={isSelected} />
    case 'mj-spacer':
      return <MjSpacerBlock node={node} isSelected={isSelected} />
    case 'mj-hero':
      return (
        <MjSectionBlock node={node} isSelected={isSelected}>
          {children}
        </MjSectionBlock>
      )
    case 'mj-wrapper':
      return (
        <div
          className={`w-full transition-all ${
            isSelected ? 'ring-2 ring-primary' : 'hover:ring-1 hover:ring-dashed hover:ring-border'
          }`}
          style={{
            backgroundColor: (node.props['background-color'] as string) || undefined,
            padding: (node.props['padding'] as string) || undefined,
          }}
        >
          {children}
        </div>
      )
    default:
      return (
        <MjFallbackBlock node={node} isSelected={isSelected} componentDef={componentDef}>
          {children}
        </MjFallbackBlock>
      )
  }
}
