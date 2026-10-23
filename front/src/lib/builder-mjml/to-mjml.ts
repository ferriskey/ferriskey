import type { BuilderNode } from '../builder-core/types.ts'
import { sanitizeEmailHtml } from './sanitize-email-html.ts'

const SANITIZED_CONTENT_TYPES = new Set(['mj-text', 'mj-button'])
const ATTRIBUTE_NAME = /^[A-Za-z][A-Za-z0-9_:-]*$/
const SELF_CLOSING = ['mj-divider', 'mj-spacer', 'mj-image']

type Sanitize = (html: string) => string

function escapeAttribute(value: string): string {
  return value
    .replace(/&/g, '&amp;')
    .replace(/"/g, '&quot;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
}

function attributesOf(values: Record<string, unknown>): string {
  return Object.entries(values)
    .filter(([name, value]) => ATTRIBUTE_NAME.test(name) && value !== undefined && value !== '')
    .map(([name, value]) => ` ${name}="${escapeAttribute(String(value))}"`)
    .join('')
}

function nodeToMjml(node: BuilderNode, sanitize: Sanitize): string {
  const allAttrs = `${attributesOf(node.props)}${attributesOf(node.styles)}`

  const rawContent = node.content ?? ''
  const content = SANITIZED_CONTENT_TYPES.has(node.type) ? sanitize(rawContent) : rawContent
  const children = node.children.map((child) => nodeToMjml(child, sanitize)).join('')

  if (SELF_CLOSING.includes(node.type) && !content && !children) {
    return `<${node.type}${allAttrs} />`
  }

  return `<${node.type}${allAttrs}>${content}${children}</${node.type}>`
}

export function treeToMjml(tree: BuilderNode[], sanitize: Sanitize = sanitizeEmailHtml): string {
  const body = tree.map((node) => nodeToMjml(node, sanitize)).join('')
  return `<mjml><mj-body>${body}</mj-body></mjml>`
}
