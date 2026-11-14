const CHILD_RENDERING_TYPES: ReadonlySet<string> = new Set([
  'container',
  'flex',
  'grid',
  'div',
  'card',
  'card-header',
  'card-content',
  'card-footer',
])

export function collectRenderedTypes(value: unknown, acc: Set<string>): void {
  if (Array.isArray(value)) {
    for (const item of value) collectRenderedTypes(item, acc)
    return
  }
  if (!value || typeof value !== 'object') return

  const node = value as { type?: unknown; children?: unknown }
  let rendersChildren = true
  if (typeof node.type === 'string') {
    acc.add(node.type)
    rendersChildren = CHILD_RENDERING_TYPES.has(node.type)
  }
  if (rendersChildren) collectRenderedTypes(node.children, acc)
}
