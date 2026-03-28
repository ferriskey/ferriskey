import mjml2html from 'mjml-browser'
import type { BuilderNode } from '../builder-core'

/**
 * Converts a builder node tree into an MJML string.
 */
export function treeToMjml(tree: BuilderNode[]): string {
  const body = tree.map(nodeToMjml).join('')
  return `<mjml><mj-body>${body}</mj-body></mjml>`
}

function nodeToMjml(node: BuilderNode): string {
  const attrs = Object.entries(node.props)
    .filter(([, v]) => v !== undefined && v !== '')
    .map(([k, v]) => ` ${k}="${String(v)}"`)
    .join('')

  const styles = Object.entries(node.styles)
    .filter(([, v]) => v !== undefined && v !== '')
    .map(([k, v]) => ` ${k}="${String(v)}"`)
    .join('')

  const allAttrs = `${attrs}${styles}`

  const content = node.content ?? ''
  const children = node.children.map(nodeToMjml).join('')

  const selfClosing = ['mj-divider', 'mj-spacer', 'mj-image']
  if (selfClosing.includes(node.type) && !content && !children) {
    return `<${node.type}${allAttrs} />`
  }

  return `<${node.type}${allAttrs}>${content}${children}</${node.type}>`
}

/**
 * Converts a builder node tree into HTML via MJML.
 */
export function treeToHtml(tree: BuilderNode[]): string {
  const mjml = treeToMjml(tree)
  const result = mjml2html(mjml, { validationLevel: 'soft' })
  return result.html
}
