import { renderToStaticMarkup } from 'react-dom/server'
import type { ReactNode } from 'react'
import type { BuilderAdapter, BuilderNode } from '../builder-core'
import { getDefaultNode, portalComponents } from './components'
import { renderPortalConfigPanel } from './config-panels/config-panel-renderer'
import { treeToReactNode } from './renderer'
import { renderVisualBlock } from './visual-blocks/visual-block-registry'

export function createPortalAdapter(): BuilderAdapter {
  return {
    components: portalComponents,

    renderConfigPanel(node, onUpdate) {
      return renderPortalConfigPanel(node, onUpdate)
    },

    renderPreview(tree: BuilderNode[]): string {
      // Builder-core's adapter contract returns a string. We rarely use this for
      // portal builders (PortalPreview consumes the tree directly via React),
      // but we keep a static-markup fallback so the contract is honored.
      return renderToStaticMarkup(treeToReactNode(tree) as ReactNode)
    },

    renderVisualBlock(node, isSelected, children) {
      const componentDef = portalComponents.find((c) => c.type === node.type)
      return renderVisualBlock(node, isSelected, children, componentDef)
    },

    getDefaultNode(type: string) {
      return getDefaultNode(type)
    },
  }
}
