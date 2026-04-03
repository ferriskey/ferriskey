import type { ReactNode } from 'react'
import type { BuilderAdapter, BuilderNode } from '../builder-core'
import { mjmlComponents, getDefaultNode } from './components'
import { renderMjmlConfigPanel } from './config-panels/config-panel-renderer'
import { treeToHtml } from './renderer'
import { renderVisualBlock } from './visual-blocks/visual-block-registry'

interface MjmlAdapterOptions {
  variables?: { name: string; description: string }[]
}

export function createMjmlAdapter(
  options: MjmlAdapterOptions = {},
): BuilderAdapter {
  return {
    components: mjmlComponents,
    variables: options.variables,

    renderConfigPanel(node, onUpdate) {
      return renderMjmlConfigPanel(node, onUpdate)
    },

    renderPreview(tree: BuilderNode[]): string {
      return treeToHtml(tree)
    },

    renderVisualBlock(node: BuilderNode, isSelected: boolean, children: ReactNode): ReactNode {
      const componentDef = mjmlComponents.find((c) => c.type === node.type)
      return renderVisualBlock(node, isSelected, children, componentDef)
    },

    getDefaultNode(type: string) {
      return getDefaultNode(type)
    },
  }
}
