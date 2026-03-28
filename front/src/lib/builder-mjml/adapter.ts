import type { BuilderAdapter, BuilderNode } from '../builder-core'
import { mjmlComponents, getDefaultNode } from './components'
import { renderMjmlConfigPanel } from './config-panels/config-panel-renderer'
import { treeToHtml } from './renderer'

interface MjmlAdapterOptions {
  variables?: { name: string; description: string }[]
}

export function createMjmlAdapter(
  options: MjmlAdapterOptions = {},
): BuilderAdapter {
  return {
    components: mjmlComponents,

    renderConfigPanel(node, onUpdate) {
      return renderMjmlConfigPanel(node, onUpdate, options.variables)
    },

    renderPreview(tree: BuilderNode[]): string {
      return treeToHtml(tree)
    },

    getDefaultNode(type: string) {
      return getDefaultNode(type)
    },
  }
}
