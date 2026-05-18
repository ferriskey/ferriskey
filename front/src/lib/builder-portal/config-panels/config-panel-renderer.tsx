import type { ReactNode } from 'react'
import type { BuilderNode } from '../../builder-core'
import { ConfigSection } from './config-section'
import { ColorField, SelectField, TextField } from './shared-fields'

type OnUpdate = (
  updates: Partial<Pick<BuilderNode, 'name' | 'props' | 'styles' | 'content'>>,
) => void

/**
 * Universal "Identity" section shown above every block's specific config.
 * Lets the author name the node so it's easy to spot in the breadcrumb.
 */
function IdentitySection({ node, onUpdate }: { node: BuilderNode; onUpdate: OnUpdate }) {
  return (
    <ConfigSection title='Identity'>
      <TextField
        label='Name'
        value={node.name ?? ''}
        onChange={(v) => onUpdate({ name: v })}
      />
    </ConfigSection>
  )
}

export function renderPortalConfigPanel(node: BuilderNode, onUpdate: OnUpdate): ReactNode {
  const updateProp = (key: string, value: string) => onUpdate({ props: { [key]: value } })
  const identity = <IdentitySection node={node} onUpdate={onUpdate} />

  switch (node.type) {
    case 'container':
      return (
        <div className='flex flex-col'>
          {identity}
          <ConfigSection title='Layout'>
            <SelectField
              label='Direction'
              value={node.props.direction as string}
              options={[
                { label: 'Column', value: 'column' },
                { label: 'Row', value: 'row' },
              ]}
              onChange={(v) => updateProp('direction', v)}
              allowEmpty={false}
            />
            <SelectField
              label='Align'
              value={node.props.align as string}
              options={[
                { label: 'Start', value: 'flex-start' },
                { label: 'Center', value: 'center' },
                { label: 'End', value: 'flex-end' },
                { label: 'Stretch', value: 'stretch' },
              ]}
              onChange={(v) => updateProp('align', v)}
              allowEmpty={false}
            />
            <TextField label='Gap' value={node.props.gap as string} onChange={(v) => updateProp('gap', v)} />
            <TextField label='Padding' value={node.props.padding as string} onChange={(v) => updateProp('padding', v)} />
          </ConfigSection>
          <ConfigSection title='Style' defaultOpen={false}>
            <ColorField
              label='Background'
              value={node.props.backgroundColor as string}
              onChange={(v) => updateProp('backgroundColor', v)}
            />
            <TextField
              label='Border radius'
              value={node.props.borderRadius as string}
              onChange={(v) => updateProp('borderRadius', v)}
            />
            <TextField label='Width' value={node.props.width as string} onChange={(v) => updateProp('width', v)} />
          </ConfigSection>
        </div>
      )

    // Legacy `flex` / `grid` block types fall through to the unified Div
    // panel; their `display` is forced by the renderer based on node.type.
    case 'flex':
    case 'grid':
    case 'div': {
      const effectiveDisplay =
        node.type === 'flex'
          ? 'flex'
          : node.type === 'grid'
            ? 'grid'
            : (node.props.display as string) || 'block'
      return (
        <div className='flex flex-col'>
          {identity}
          <ConfigSection title='Display'>
            <SelectField
              label='Display'
              value={(node.props.display as string) || 'block'}
              options={[
                { label: 'Block', value: 'block' },
                { label: 'Flex', value: 'flex' },
                { label: 'Grid', value: 'grid' },
                { label: 'Inline', value: 'inline' },
                { label: 'Inline block', value: 'inline-block' },
                { label: 'None', value: 'none' },
              ]}
              onChange={(v) => updateProp('display', v)}
              allowEmpty={false}
            />
          </ConfigSection>

          {effectiveDisplay === 'flex' && (
            <ConfigSection title='Flex'>
              <SelectField
                label='Direction'
                value={node.props.direction as string}
                options={[
                  { label: 'Row', value: 'row' },
                  { label: 'Row reverse', value: 'row-reverse' },
                  { label: 'Column', value: 'column' },
                  { label: 'Column reverse', value: 'column-reverse' },
                ]}
                onChange={(v) => updateProp('direction', v)}
                allowEmpty={false}
              />
              <SelectField
                label='Wrap'
                value={node.props.wrap as string}
                options={[
                  { label: 'No wrap', value: 'nowrap' },
                  { label: 'Wrap', value: 'wrap' },
                  { label: 'Wrap reverse', value: 'wrap-reverse' },
                ]}
                onChange={(v) => updateProp('wrap', v)}
                allowEmpty={false}
              />
              <SelectField
                label='Justify content'
                value={node.props.justifyContent as string}
                options={[
                  { label: 'Start', value: 'flex-start' },
                  { label: 'End', value: 'flex-end' },
                  { label: 'Center', value: 'center' },
                  { label: 'Space between', value: 'space-between' },
                  { label: 'Space around', value: 'space-around' },
                  { label: 'Space evenly', value: 'space-evenly' },
                ]}
                onChange={(v) => updateProp('justifyContent', v)}
                allowEmpty={false}
              />
              <SelectField
                label='Align items'
                value={node.props.alignItems as string}
                options={[
                  { label: 'Start', value: 'flex-start' },
                  { label: 'End', value: 'flex-end' },
                  { label: 'Center', value: 'center' },
                  { label: 'Baseline', value: 'baseline' },
                  { label: 'Stretch', value: 'stretch' },
                ]}
                onChange={(v) => updateProp('alignItems', v)}
                allowEmpty={false}
              />
              <SelectField
                label='Align content'
                value={node.props.alignContent as string}
                options={[
                  { label: 'Start', value: 'flex-start' },
                  { label: 'End', value: 'flex-end' },
                  { label: 'Center', value: 'center' },
                  { label: 'Space between', value: 'space-between' },
                  { label: 'Space around', value: 'space-around' },
                  { label: 'Stretch', value: 'stretch' },
                ]}
                onChange={(v) => updateProp('alignContent', v)}
                allowEmpty={false}
              />
            </ConfigSection>
          )}

          {effectiveDisplay === 'grid' && (
            <ConfigSection title='Grid'>
              <TextField
                label='Template columns'
                value={node.props.templateColumns as string}
                onChange={(v) => updateProp('templateColumns', v)}
              />
              <TextField
                label='Template rows'
                value={node.props.templateRows as string}
                onChange={(v) => updateProp('templateRows', v)}
              />
              <TextField
                label='Column gap'
                value={node.props.columnGap as string}
                onChange={(v) => updateProp('columnGap', v)}
              />
              <TextField
                label='Row gap'
                value={node.props.rowGap as string}
                onChange={(v) => updateProp('rowGap', v)}
              />
              <SelectField
                label='Justify items'
                value={node.props.justifyItems as string}
                options={[
                  { label: 'Start', value: 'start' },
                  { label: 'End', value: 'end' },
                  { label: 'Center', value: 'center' },
                  { label: 'Stretch', value: 'stretch' },
                ]}
                onChange={(v) => updateProp('justifyItems', v)}
                allowEmpty={false}
              />
              <SelectField
                label='Align items'
                value={node.props.alignItems as string}
                options={[
                  { label: 'Start', value: 'start' },
                  { label: 'End', value: 'end' },
                  { label: 'Center', value: 'center' },
                  { label: 'Stretch', value: 'stretch' },
                ]}
                onChange={(v) => updateProp('alignItems', v)}
                allowEmpty={false}
              />
              <SelectField
                label='Auto flow'
                value={node.props.autoFlow as string}
                options={[
                  { label: 'Row', value: 'row' },
                  { label: 'Column', value: 'column' },
                  { label: 'Row dense', value: 'row dense' },
                  { label: 'Column dense', value: 'column dense' },
                ]}
                onChange={(v) => updateProp('autoFlow', v)}
                allowEmpty={false}
              />
            </ConfigSection>
          )}

          <ConfigSection title='Position' defaultOpen={false}>
            <SelectField
              label='Position'
              value={node.props.position as string}
              options={[
                { label: 'Static', value: 'static' },
                { label: 'Relative', value: 'relative' },
                { label: 'Absolute', value: 'absolute' },
                { label: 'Fixed', value: 'fixed' },
                { label: 'Sticky', value: 'sticky' },
              ]}
              onChange={(v) => updateProp('position', v)}
              allowEmpty={false}
            />
            <TextField label='Top' value={node.props.top as string} onChange={(v) => updateProp('top', v)} />
            <TextField label='Right' value={node.props.right as string} onChange={(v) => updateProp('right', v)} />
            <TextField label='Bottom' value={node.props.bottom as string} onChange={(v) => updateProp('bottom', v)} />
            <TextField label='Left' value={node.props.left as string} onChange={(v) => updateProp('left', v)} />
            <TextField label='z-index' value={node.props.zIndex as string} onChange={(v) => updateProp('zIndex', v)} />
          </ConfigSection>
          <ConfigSection title='Size' defaultOpen={false}>
            <TextField label='Width' value={node.props.width as string} onChange={(v) => updateProp('width', v)} />
            <TextField label='Height' value={node.props.height as string} onChange={(v) => updateProp('height', v)} />
            <TextField
              label='Min width'
              value={node.props.minWidth as string}
              onChange={(v) => updateProp('minWidth', v)}
            />
            <TextField
              label='Max width'
              value={node.props.maxWidth as string}
              onChange={(v) => updateProp('maxWidth', v)}
            />
            <TextField
              label='Min height'
              value={node.props.minHeight as string}
              onChange={(v) => updateProp('minHeight', v)}
            />
            <TextField
              label='Max height'
              value={node.props.maxHeight as string}
              onChange={(v) => updateProp('maxHeight', v)}
            />
            <SelectField
              label='Overflow'
              value={node.props.overflow as string}
              options={[
                { label: 'Visible', value: 'visible' },
                { label: 'Hidden', value: 'hidden' },
                { label: 'Scroll', value: 'scroll' },
                { label: 'Auto', value: 'auto' },
              ]}
              onChange={(v) => updateProp('overflow', v)}
              allowEmpty={false}
            />
          </ConfigSection>
          <ConfigSection title='Spacing' defaultOpen={false}>
            <TextField label='Padding' value={node.props.padding as string} onChange={(v) => updateProp('padding', v)} />
            <TextField label='Margin' value={node.props.margin as string} onChange={(v) => updateProp('margin', v)} />
            <TextField label='Gap' value={node.props.gap as string} onChange={(v) => updateProp('gap', v)} />
          </ConfigSection>
          <ConfigSection title='Style' defaultOpen={false}>
            <ColorField
              label='Background'
              value={node.props.backgroundColor as string}
              onChange={(v) => updateProp('backgroundColor', v)}
            />
            <TextField
              label='Border radius'
              value={node.props.borderRadius as string}
              onChange={(v) => updateProp('borderRadius', v)}
            />
          </ConfigSection>
        </div>
      )
    }

    case 'heading':
      return (
        <div className='flex flex-col'>
          {identity}
          <ConfigSection title='Typography'>
            <SelectField
              label='Level'
              value={node.props.level as string}
              options={[
                { label: 'H1', value: '1' },
                { label: 'H2', value: '2' },
                { label: 'H3', value: '3' },
                { label: 'H4', value: '4' },
              ]}
              onChange={(v) => updateProp('level', v)}
              allowEmpty={false}
            />
            <ColorField label='Color' value={node.props.color as string} onChange={(v) => updateProp('color', v)} />
            <TextField label='Font size' value={node.props.fontSize as string} onChange={(v) => updateProp('fontSize', v)} />
            <SelectField
              label='Weight'
              value={node.props.fontWeight as string}
              options={[
                { label: 'Regular', value: '400' },
                { label: 'Medium', value: '500' },
                { label: 'Semibold', value: '600' },
                { label: 'Bold', value: '700' },
              ]}
              onChange={(v) => updateProp('fontWeight', v)}
            />
            <SelectField
              label='Align'
              value={node.props.textAlign as string}
              options={[
                { label: 'Left', value: 'left' },
                { label: 'Center', value: 'center' },
                { label: 'Right', value: 'right' },
              ]}
              onChange={(v) => updateProp('textAlign', v)}
              allowEmpty={false}
            />
          </ConfigSection>
        </div>
      )

    case 'text':
      return (
        <div className='flex flex-col'>
          {identity}
          <ConfigSection title='Typography'>
            <ColorField label='Color' value={node.props.color as string} onChange={(v) => updateProp('color', v)} />
            <TextField label='Font size' value={node.props.fontSize as string} onChange={(v) => updateProp('fontSize', v)} />
            <SelectField
              label='Weight'
              value={node.props.fontWeight as string}
              options={[
                { label: 'Regular', value: '400' },
                { label: 'Medium', value: '500' },
                { label: 'Semibold', value: '600' },
                { label: 'Bold', value: '700' },
              ]}
              onChange={(v) => updateProp('fontWeight', v)}
            />
            <TextField label='Line height' value={node.props.lineHeight as string} onChange={(v) => updateProp('lineHeight', v)} />
            <SelectField
              label='Align'
              value={node.props.textAlign as string}
              options={[
                { label: 'Left', value: 'left' },
                { label: 'Center', value: 'center' },
                { label: 'Right', value: 'right' },
              ]}
              onChange={(v) => updateProp('textAlign', v)}
              allowEmpty={false}
            />
          </ConfigSection>
        </div>
      )

    case 'image':
      return (
        <div className='flex flex-col'>
          {identity}
          <ConfigSection title='Source'>
            <TextField label='URL' value={node.props.src as string} onChange={(v) => updateProp('src', v)} />
            <TextField label='Alt' value={node.props.alt as string} onChange={(v) => updateProp('alt', v)} />
          </ConfigSection>
          <ConfigSection title='Style' defaultOpen={false}>
            <TextField label='Width' value={node.props.width as string} onChange={(v) => updateProp('width', v)} />
            <TextField label='Height' value={node.props.height as string} onChange={(v) => updateProp('height', v)} />
            <TextField
              label='Border radius'
              value={node.props.borderRadius as string}
              onChange={(v) => updateProp('borderRadius', v)}
            />
            <SelectField
              label='Align'
              value={node.props.align as string}
              options={[
                { label: 'Left', value: 'left' },
                { label: 'Center', value: 'center' },
                { label: 'Right', value: 'right' },
              ]}
              onChange={(v) => updateProp('align', v)}
              allowEmpty={false}
            />
          </ConfigSection>
        </div>
      )

    case 'spacer':
      return (
        <div className='flex flex-col gap-3 pt-2'>
          <TextField label='Height' value={node.props.height as string} onChange={(v) => updateProp('height', v)} />
        </div>
      )

    case 'divider':
      return (
        <div className='flex flex-col gap-3 pt-2'>
          <ColorField label='Color' value={node.props.color as string} onChange={(v) => updateProp('color', v)} />
          <TextField label='Thickness' value={node.props.thickness as string} onChange={(v) => updateProp('thickness', v)} />
          <TextField label='Width' value={node.props.width as string} onChange={(v) => updateProp('width', v)} />
        </div>
      )

    case 'button':
      return (
        <div className='flex flex-col'>
          {identity}
          <ConfigSection title='Action'>
            <TextField label='Link URL' value={node.props.href as string} onChange={(v) => updateProp('href', v)} />
          </ConfigSection>
          <ConfigSection title='Style' defaultOpen={false}>
            <SelectField
              label='Variant'
              value={node.props.variant as string}
              options={[
                { label: 'Primary', value: 'primary' },
                { label: 'Secondary', value: 'secondary' },
                { label: 'Outline', value: 'outline' },
              ]}
              onChange={(v) => updateProp('variant', v)}
              allowEmpty={false}
            />
            <SelectField
              label='Width'
              value={node.props.fullWidth as string}
              options={[
                { label: 'Full', value: 'true' },
                { label: 'Auto', value: 'false' },
              ]}
              onChange={(v) => updateProp('fullWidth', v)}
              allowEmpty={false}
            />
          </ConfigSection>
        </div>
      )

    case 'submit_button':
      // Protected block: its action is hard-wired to submit the form.
      // Only visual style is configurable.
      return (
        <div className='flex flex-col'>
          {identity}
          <div className='border-b border-border bg-muted/40 px-3 py-2 text-[11px] text-muted-foreground'>
            Submit action is locked — this button always submits the page's form.
          </div>
          <ConfigSection title='Style'>
            <SelectField
              label='Variant'
              value={node.props.variant as string}
              options={[
                { label: 'Primary', value: 'primary' },
                { label: 'Secondary', value: 'secondary' },
                { label: 'Outline', value: 'outline' },
              ]}
              onChange={(v) => updateProp('variant', v)}
              allowEmpty={false}
            />
            <SelectField
              label='Width'
              value={node.props.fullWidth as string}
              options={[
                { label: 'Full', value: 'true' },
                { label: 'Auto', value: 'false' },
              ]}
              onChange={(v) => updateProp('fullWidth', v)}
              allowEmpty={false}
            />
          </ConfigSection>
        </div>
      )

    case 'email_input':
    case 'password_input':
    case 'totp_input': {
      const lockedHint =
        node.type === 'email_input'
          ? 'Email input — the HTML type and field name are locked to email.'
          : node.type === 'password_input'
            ? 'Password input — the HTML type and field name are locked to password.'
            : 'TOTP input — the HTML type is text and the field name is locked to totp.'
      return (
        <div className='flex flex-col'>
          {identity}
          <div className='border-b border-border bg-muted/40 px-3 py-2 text-[11px] text-muted-foreground'>
            {lockedHint}
          </div>
          <ConfigSection title='Field'>
            <TextField label='Label' value={node.props.label as string} onChange={(v) => updateProp('label', v)} />
            <TextField
              label='Placeholder'
              value={node.props.placeholder as string}
              onChange={(v) => updateProp('placeholder', v)}
            />
            <TextField
              label='Helper text'
              value={node.props.helperText as string}
              onChange={(v) => updateProp('helperText', v)}
            />
          </ConfigSection>
        </div>
      )
    }

    case 'input':
      return (
        <div className='flex flex-col'>
          {identity}
          <ConfigSection title='Field'>
            <TextField label='Label' value={node.props.label as string} onChange={(v) => updateProp('label', v)} />
            <TextField
              label='Placeholder'
              value={node.props.placeholder as string}
              onChange={(v) => updateProp('placeholder', v)}
            />
            <SelectField
              label='Type'
              value={node.props.type as string}
              options={[
                { label: 'Text', value: 'text' },
                { label: 'Email', value: 'email' },
                { label: 'Password', value: 'password' },
              ]}
              onChange={(v) => updateProp('type', v)}
              allowEmpty={false}
            />
            <TextField
              label='Helper text'
              value={node.props.helperText as string}
              onChange={(v) => updateProp('helperText', v)}
            />
          </ConfigSection>
        </div>
      )

    case 'page-content':
      return (
        <div className='flex flex-col'>
          {identity}
          <div className='px-3 py-2 text-xs text-muted-foreground'>
            The page content slot is replaced at runtime by the page using this layout. No configuration needed.
          </div>
        </div>
      )

    default:
      return (
        <div className='flex flex-col'>
          {identity}
          <div className='px-3 py-2 text-xs text-muted-foreground'>
            No configuration for {node.type}
          </div>
        </div>
      )
  }
}
