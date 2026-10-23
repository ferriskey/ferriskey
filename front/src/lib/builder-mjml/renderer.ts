import mjml2html from 'mjml-browser'
import type { BuilderNode } from '../builder-core'
import { treeToMjml } from './to-mjml'

export { treeToMjml }

/**
 * Converts a builder node tree into HTML via MJML.
 */
export async function treeToHtml(tree: BuilderNode[]): Promise<string> {
  const mjml = treeToMjml(tree)
  const result = await mjml2html(mjml, { validationLevel: 'soft' })
  return result.html
}
