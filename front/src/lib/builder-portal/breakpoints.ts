import type { CSSProperties } from 'react'
import { BREAKPOINTS, type Breakpoint, type BuilderNode } from '../builder-core'
import {
  buttonStyle,
  containerStyle,
  divStyle,
  headingStyle,
  imageStyle,
  inputFieldStyle,
  orderStyle,
  textStyle,
} from './renderer'

/** Tailwind-default min-width thresholds (in px). */
export const BREAKPOINT_WIDTHS: Record<Breakpoint, number> = {
  sm: 640,
  md: 768,
  lg: 1024,
  xl: 1280,
}

/**
 * Returns the CSSProperties for a node given a (potentially merged) props
 * map — same logic the canvas uses for selection-state rendering. New block
 * types should be added here to participate in breakpoint overrides.
 */
function styleForNodeWithProps(
  type: string,
  props: Record<string, unknown>,
): CSSProperties {
  // Shape a temporary node so we can reuse the existing style helpers.
  const fake: BuilderNode = { id: '', type, props, styles: {}, children: [] }
  switch (type) {
    case 'container':
      return containerStyle(fake)
    case 'flex':
    case 'grid':
    case 'div':
      return divStyle(fake)
    case 'heading':
      return headingStyle(fake)
    case 'text':
      return textStyle(fake)
    case 'image':
      // Block-level CSS targets the outer wrapper (it carries `data-fk-id`),
      // not the inner <img>. Merge wrapper-only props (`order`) on top of
      // the img style so `order` overrides flow into the @media rules.
      return { ...imageStyle(fake), ...orderStyle(fake) }
    case 'button':
    case 'submit_button':
      return buttonStyle(fake)
    case 'input':
    case 'email_input':
    case 'password_input':
    case 'totp_input':
      // Same as `image`: the `data-fk-id` selector hits the wrapper, where
      // `order` lives. The field style is the inner <input>.
      return { ...inputFieldStyle(), ...orderStyle(fake) }
    default:
      return {}
  }
}

const CAMEL_TO_KEBAB_RE = /[A-Z]/g
function cssKey(camel: string): string {
  return camel.replace(CAMEL_TO_KEBAB_RE, (m) => `-${m.toLowerCase()}`)
}

function styleToCssText(style: CSSProperties): string {
  return Object.entries(style)
    .filter(([, v]) => v !== undefined && v !== '' && v !== null)
    .map(([k, v]) => `${cssKey(k)}: ${v} !important;`)
    .join(' ')
}

/**
 * Walks the tree and emits one block of CSS rules per node-with-overrides.
 * Each block looks like:
 *
 *   @media (min-width: 768px) {
 *     [data-fk-id="abc"] { padding: 32px !important; … }
 *   }
 *
 * The `data-fk-id` attribute is added to every block by the renderer / canvas
 * so these selectors hit. `!important` ensures the media-query layer wins
 * over the inline base style.
 */
export function generateBreakpointCss(tree: BuilderNode[]): string {
  const rules: string[] = []
  walk(tree, (node) => {
    if (!node.breakpoints) return
    // Mobile-first: accumulate overrides as we walk up the breakpoints, and
    // diff each layer against the previous accumulated state — not against
    // the base. That way, if `md` happens to restore the base value while
    // `sm` shifted it, we still emit a `@media (min-width: 768px)` rule that
    // overrides the lingering `sm` declaration in the cascade.
    let accumulated: Record<string, unknown> = { ...node.props }
    for (const bp of BREAKPOINTS) {
      const override = node.breakpoints[bp]
      if (!override || Object.keys(override).length === 0) continue
      const nextAccumulated = { ...accumulated, ...override }
      const prevStyle = styleForNodeWithProps(node.type, accumulated)
      const style = styleForNodeWithProps(node.type, nextAccumulated)
      const diff: CSSProperties = {}
      for (const k of Object.keys(style) as (keyof CSSProperties)[]) {
        if (style[k] !== prevStyle[k]) {
          // Assigning string to a possibly non-string CSSProperty key.
          ;(diff as Record<string, unknown>)[k as string] = style[k]
        }
      }
      accumulated = nextAccumulated
      const body = styleToCssText(diff)
      if (!body) continue
      // We target both `data-fk-id` (the rendered block — what runtime hits)
      // and `data-sortable-id` (dnd-kit's wrapper that becomes the real
      // flex/grid item inside the editor canvas). Item-positioning props
      // (`order`, `align-self`, etc.) only take effect on the wrapper in the
      // editor; container props (`grid-template-columns`, `gap`, …) are
      // no-ops on the wrapper (no `display: grid` there), so emitting both
      // selectors is safe.
      rules.push(
        `@media (min-width: ${BREAKPOINT_WIDTHS[bp]}px) { [data-fk-id="${node.id}"], [data-sortable-id="${node.id}"] { ${body} } }`,
      )
    }
  })
  return rules.join('\n')
}

function walk(tree: BuilderNode[], visit: (node: BuilderNode) => void): void {
  for (const node of tree) {
    visit(node)
    if (node.children.length > 0) walk(node.children, visit)
  }
}
