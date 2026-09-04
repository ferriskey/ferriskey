import type { BuilderNode } from '../../builder-core'
import { ColorField } from './shared-fields'
import { ConfigSection } from './config-section'

/**
 * Colour overrides shared by the button blocks.
 *
 * Each field is optional: left empty, the button keeps the colour its variant
 * takes from the theme, so branding stays centralised by default and only the
 * buttons an author deliberately singles out break away from it.
 */
export function ButtonColorSection({
  node,
  updateProp,
}: {
  node: BuilderNode
  updateProp: (key: string, value: string) => void
}) {
  return (
    <ConfigSection title='Colors' defaultOpen={false}>
      <div className='px-3 pb-1 text-[11px] text-muted-foreground'>
        Leave empty to follow the theme.
      </div>
      <ColorField
        label='Background'
        value={node.props.backgroundColor as string}
        onChange={(v) => updateProp('backgroundColor', v)}
      />
      <ColorField
        label='Label'
        value={node.props.color as string}
        onChange={(v) => updateProp('color', v)}
      />
      <ColorField
        label='Border'
        value={node.props.borderColor as string}
        onChange={(v) => updateProp('borderColor', v)}
      />
    </ConfigSection>
  )
}
