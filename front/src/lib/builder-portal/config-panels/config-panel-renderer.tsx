import { ChevronDown, ChevronUp } from 'lucide-react'
import type { ReactNode } from 'react'
import { findNodePath, useBuilderContext, type BuilderNode } from '../../builder-core'
import { ConfigSection } from './config-section'
import { DimensionInput } from './dimension-input'
import { LinkedSidesInput } from './linked-sides-input'
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

/**
 * Universal "Flex / Grid item" section. CSS `order` only takes effect when
 * this node's parent is a flex/grid container, so we only surface the
 * section in that context. The Up / Down buttons decrement / increment the
 * `order` prop at whatever breakpoint the user is editing — so the cascade
 * + per-bp routing already in place mean you can have base order=0, md
 * order=2, xl order=-1, etc.
 */
function ItemSection({ node, onUpdate }: { node: BuilderNode; onUpdate: OnUpdate }) {
  const { tree } = useBuilderContext()
  const path = findNodePath(tree, node.id)
  const parent = path.length >= 2 ? path[path.length - 2] : null

  // `container` is always flex; legacy `flex`/`grid` block types coerce.
  // For a real `div`, look at the declared display prop.
  const parentDisplay = parent
    ? parent.type === 'container' || parent.type === 'flex'
      ? 'flex'
      : parent.type === 'grid'
        ? 'grid'
        : ((parent.props.display as string) ?? '')
    : ''
  if (parentDisplay !== 'flex' && parentDisplay !== 'grid') return null

  // `node.props.order` here is the *effective* cascaded value at the editing
  // breakpoint (config-panel.tsx merges base ← sm ← md ← ... up to the
  // active bp before passing it in), so the buttons act on what the user is
  // actually seeing. The write is then routed by the parent ConfigPanel to
  // the editing bp's override layer.
  const rawOrder = (node.props.order as string) ?? ''
  const currentOrder = rawOrder === '' ? 0 : Number(rawOrder)
  const setOrder = (next: number) => onUpdate({ props: { order: String(next) } })

  return (
    <ConfigSection title='Flex / Grid item' defaultOpen>
      <div className='flex items-center gap-1.5 pb-2'>
        <button
          type='button'
          onClick={() => setOrder(currentOrder - 1)}
          className='flex h-7 flex-1 items-center justify-center gap-1 rounded-md border border-border bg-background text-xs text-muted-foreground transition-colors hover:bg-muted'
          title='Move earlier (order − 1)'
        >
          <ChevronUp size={14} /> Earlier
        </button>
        <button
          type='button'
          onClick={() => setOrder(currentOrder + 1)}
          className='flex h-7 flex-1 items-center justify-center gap-1 rounded-md border border-border bg-background text-xs text-muted-foreground transition-colors hover:bg-muted'
          title='Move later (order + 1)'
        >
          <ChevronDown size={14} /> Later
        </button>
      </div>
      <p className='-mt-1 pb-1 text-[10px] leading-tight text-muted-foreground'>
        Adjusts the CSS <code>order</code> of this item at the active
        breakpoint — siblings with smaller <code>order</code> render first.
        Default is <code>0</code>; use negatives to move ahead of un-ordered
        siblings.
      </p>
      <TextField
        label='Order'
        value={rawOrder}
        onChange={(v) => onUpdate({ props: { order: v } })}
      />
    </ConfigSection>
  )
}

export function renderPortalConfigPanel(node: BuilderNode, onUpdate: OnUpdate): ReactNode {
  const item = <ItemSection node={node} onUpdate={onUpdate} />
  // page-content has no real config and shouldn't show the Order field —
  // it's slotted by the runtime layout, never a flex/grid item the admin
  // arranges.
  return (
    <>
      {renderPortalConfigPanelInner(node, onUpdate)}
      {node.type !== 'page-content' && item}
    </>
  )
}

function renderPortalConfigPanelInner(node: BuilderNode, onUpdate: OnUpdate): ReactNode {
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
            <DimensionInput label='Gap' value={node.props.gap as string} onChange={(v) => updateProp('gap', v)} />
            <LinkedSidesInput label='Padding' value={node.props.padding as string} onChange={(v) => updateProp('padding', v)} />
          </ConfigSection>
          <ConfigSection title='Style' defaultOpen={false}>
            <ColorField
              label='Background'
              value={node.props.backgroundColor as string}
              onChange={(v) => updateProp('backgroundColor', v)}
            />
            <LinkedSidesInput
              label='Border radius'
              value={node.props.borderRadius as string}
              onChange={(v) => updateProp('borderRadius', v)}
              mode='corners'
            />
            <DimensionInput label='Width' value={node.props.width as string} onChange={(v) => updateProp('width', v)} />
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
              <SelectField
                label='Direction'
                value={(node.props.gridDirection as string) || 'ltr'}
                options={[
                  { label: 'Left to right', value: 'ltr' },
                  { label: 'Right to left (reverse columns)', value: 'rtl' },
                ]}
                onChange={(v) => updateProp('gridDirection', v)}
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
            <DimensionInput label='Top' value={node.props.top as string} onChange={(v) => updateProp('top', v)} />
            <DimensionInput label='Right' value={node.props.right as string} onChange={(v) => updateProp('right', v)} />
            <DimensionInput label='Bottom' value={node.props.bottom as string} onChange={(v) => updateProp('bottom', v)} />
            <DimensionInput label='Left' value={node.props.left as string} onChange={(v) => updateProp('left', v)} />
            <TextField label='z-index' value={node.props.zIndex as string} onChange={(v) => updateProp('zIndex', v)} />
          </ConfigSection>
          <ConfigSection title='Size' defaultOpen={false}>
            <DimensionInput label='Width' value={node.props.width as string} onChange={(v) => updateProp('width', v)} />
            <DimensionInput label='Height' value={node.props.height as string} onChange={(v) => updateProp('height', v)} />
            <DimensionInput
              label='Min width'
              value={node.props.minWidth as string}
              onChange={(v) => updateProp('minWidth', v)}
            />
            <DimensionInput
              label='Max width'
              value={node.props.maxWidth as string}
              onChange={(v) => updateProp('maxWidth', v)}
            />
            <DimensionInput
              label='Min height'
              value={node.props.minHeight as string}
              onChange={(v) => updateProp('minHeight', v)}
            />
            <DimensionInput
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
            <LinkedSidesInput label='Padding' value={node.props.padding as string} onChange={(v) => updateProp('padding', v)} />
            <LinkedSidesInput label='Margin' value={node.props.margin as string} onChange={(v) => updateProp('margin', v)} />
            <DimensionInput label='Gap' value={node.props.gap as string} onChange={(v) => updateProp('gap', v)} />
          </ConfigSection>
          <ConfigSection title='Style' defaultOpen={false}>
            <ColorField
              label='Background'
              value={node.props.backgroundColor as string}
              onChange={(v) => updateProp('backgroundColor', v)}
            />
            <LinkedSidesInput
              label='Border radius'
              value={node.props.borderRadius as string}
              onChange={(v) => updateProp('borderRadius', v)}
              mode='corners'
            />
          </ConfigSection>
        </div>
      )
    }

    case 'card':
      return (
        <div className='flex flex-col'>
          {identity}
          <ConfigSection title='Layout'>
            <SelectField
              label='Alignment'
              value={(node.props.align as string) || 'center'}
              options={[
                { label: 'Left', value: 'left' },
                { label: 'Center', value: 'center' },
                { label: 'Right', value: 'right' },
              ]}
              onChange={(v) => updateProp('align', v)}
              allowEmpty={false}
            />
            <DimensionInput
              label='Max width'
              value={node.props.maxWidth as string}
              onChange={(v) => updateProp('maxWidth', v)}
            />
            <LinkedSidesInput
              label='Padding'
              value={node.props.padding as string}
              onChange={(v) => updateProp('padding', v)}
            />
            <DimensionInput
              label='Gap'
              value={node.props.gap as string}
              onChange={(v) => updateProp('gap', v)}
            />
          </ConfigSection>
          <ConfigSection title='Style' defaultOpen={false}>
            <ColorField
              label='Background'
              value={node.props.backgroundColor as string}
              onChange={(v) => updateProp('backgroundColor', v)}
            />
            <LinkedSidesInput
              label='Border radius'
              value={node.props.borderRadius as string}
              onChange={(v) => updateProp('borderRadius', v)}
              mode='corners'
            />
            <ColorField
              label='Border color'
              value={node.props.borderColor as string}
              onChange={(v) => updateProp('borderColor', v)}
            />
            <TextField
              label='Box shadow'
              value={node.props.boxShadow as string}
              onChange={(v) => updateProp('boxShadow', v)}
            />
          </ConfigSection>
        </div>
      )

    case 'card-header':
      return (
        <div className='flex flex-col'>
          {identity}
          <ConfigSection title='Layout'>
            <SelectField
              label='Text align'
              value={(node.props.textAlign as string) || 'center'}
              options={[
                { label: 'Left', value: 'left' },
                { label: 'Center', value: 'center' },
                { label: 'Right', value: 'right' },
              ]}
              onChange={(v) => updateProp('textAlign', v)}
              allowEmpty={false}
            />
            <DimensionInput
              label='Gap'
              value={node.props.gap as string}
              onChange={(v) => updateProp('gap', v)}
            />
          </ConfigSection>
        </div>
      )

    case 'card-content':
      return (
        <div className='flex flex-col'>
          {identity}
          <ConfigSection title='Layout'>
            <DimensionInput
              label='Gap'
              value={node.props.gap as string}
              onChange={(v) => updateProp('gap', v)}
            />
          </ConfigSection>
        </div>
      )

    case 'card-footer':
      return (
        <div className='flex flex-col'>
          {identity}
          <ConfigSection title='Layout'>
            <SelectField
              label='Direction'
              value={(node.props.direction as string) || 'row'}
              options={[
                { label: 'Row', value: 'row' },
                { label: 'Column', value: 'column' },
              ]}
              onChange={(v) => updateProp('direction', v)}
              allowEmpty={false}
            />
            <SelectField
              label='Justify'
              value={(node.props.justifyContent as string) || 'flex-end'}
              options={[
                { label: 'Start', value: 'flex-start' },
                { label: 'Center', value: 'center' },
                { label: 'End', value: 'flex-end' },
                { label: 'Space between', value: 'space-between' },
              ]}
              onChange={(v) => updateProp('justifyContent', v)}
              allowEmpty={false}
            />
            <DimensionInput
              label='Gap'
              value={node.props.gap as string}
              onChange={(v) => updateProp('gap', v)}
            />
          </ConfigSection>
        </div>
      )

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
            <DimensionInput label='Width' value={node.props.width as string} onChange={(v) => updateProp('width', v)} />
            <DimensionInput label='Height' value={node.props.height as string} onChange={(v) => updateProp('height', v)} />
            <LinkedSidesInput
              label='Border radius'
              value={node.props.borderRadius as string}
              onChange={(v) => updateProp('borderRadius', v)}
              mode='corners'
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
          <DimensionInput label='Height' value={node.props.height as string} onChange={(v) => updateProp('height', v)} />
        </div>
      )

    case 'divider':
      return (
        <div className='flex flex-col gap-3 pt-2'>
          <ColorField label='Color' value={node.props.color as string} onChange={(v) => updateProp('color', v)} />
          <DimensionInput label='Thickness' value={node.props.thickness as string} onChange={(v) => updateProp('thickness', v)} />
          <DimensionInput label='Width' value={node.props.width as string} onChange={(v) => updateProp('width', v)} />
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
              label='Alignment'
              value={(node.props.align as string) || 'center'}
              options={[
                { label: 'Left', value: 'left' },
                { label: 'Center', value: 'center' },
                { label: 'Right', value: 'right' },
              ]}
              onChange={(v) => updateProp('align', v)}
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
              label='Alignment'
              value={(node.props.align as string) || 'center'}
              options={[
                { label: 'Left', value: 'left' },
                { label: 'Center', value: 'center' },
                { label: 'Right', value: 'right' },
              ]}
              onChange={(v) => updateProp('align', v)}
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

    case 'username_input':
    case 'first_name_input':
    case 'last_name_input':
    case 'email_input':
    case 'password_input':
    case 'password_confirm_input':
    case 'totp_input': {
      const LOCKED_HINTS: Record<string, string> = {
        email_input: 'Email input — the HTML type and field name are locked to email.',
        password_input: 'Password input — the HTML type and field name are locked to password.',
        password_confirm_input:
          'Confirm-password input — submit handlers compare it against the password field; field name is locked to password_confirm.',
        totp_input:
          'TOTP input — rendered as a segmented digit-by-digit OTP field. The field name is locked to totp; the slot count is configurable below.',
        username_input: 'Username input — the field name is locked to username.',
        first_name_input: 'First name input — the field name is locked to first_name.',
        last_name_input: 'Last name input — the field name is locked to last_name.',
      }
      const lockedHint = LOCKED_HINTS[node.type] ?? ''
      return (
        <div className='flex flex-col'>
          {identity}
          <div className='border-b border-border bg-muted/40 px-3 py-2 text-[11px] text-muted-foreground'>
            {lockedHint}
          </div>
          <ConfigSection title='Field'>
            <TextField label='Label' value={node.props.label as string} onChange={(v) => updateProp('label', v)} />
            {node.type !== 'totp_input' && (
              <TextField
                label='Placeholder'
                value={node.props.placeholder as string}
                onChange={(v) => updateProp('placeholder', v)}
              />
            )}
            <TextField
              label='Helper text'
              value={node.props.helperText as string}
              onChange={(v) => updateProp('helperText', v)}
            />
            {node.type === 'totp_input' && (
              <TextField
                label='Slot count'
                value={(node.props.length as string) ?? '6'}
                onChange={(v) => updateProp('length', v)}
                placeholder='6'
              />
            )}
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

    case 'magic_link_button':
    case 'passkey_button':
      // Hard-wired action: clicking triggers the magic-link or passkey
      // flow via the host page's `data-fk-action` handler. Only visual
      // style is editable here — same pattern as submit_button.
      return (
        <div className='flex flex-col'>
          {identity}
          <div className='border-b border-border bg-muted/40 px-3 py-2 text-[11px] text-muted-foreground'>
            {node.type === 'magic_link_button'
              ? 'Magic link action is locked — the portal sends a one-time sign-in email when clicked. Colors and border width are configured in Theme \u2192 Buttons.'
              : 'Passkey action is locked — the portal triggers the browser passkey prompt when clicked. Colors and border width are configured in Theme \u2192 Buttons.'}
          </div>
          <ConfigSection title='Layout'>
            <SelectField
              label='Alignment'
              value={(node.props.align as string) || 'center'}
              options={[
                { label: 'Left', value: 'left' },
                { label: 'Center', value: 'center' },
                { label: 'Right', value: 'right' },
              ]}
              onChange={(v) => updateProp('align', v)}
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

    case 'identity_providers':
      return (
        <div className='flex flex-col'>
          {identity}
          <div className='border-b border-border bg-muted/40 px-3 py-2 text-[11px] text-muted-foreground'>
            Identity providers — the list is populated at runtime from the realm's configured providers. Only the labels are editable.
          </div>
          <ConfigSection title='Labels'>
            <TextField
              label='Separator label'
              value={node.props.separatorLabel as string}
              onChange={(v) => updateProp('separatorLabel', v)}
            />
            <TextField
              label='Button label prefix'
              value={node.props.buttonLabel as string}
              onChange={(v) => updateProp('buttonLabel', v)}
            />
          </ConfigSection>
        </div>
      )

    case 'forgot_password_link':
    case 'back_to_login_link':
    case 'register_link':
      return (
        <div className='flex flex-col'>
          {identity}
          <div className='border-b border-border bg-muted/40 px-3 py-2 text-[11px] text-muted-foreground'>
            {node.type === 'forgot_password_link'
              ? 'Renders as a link to the realm\u2019s forgot-password page. Styling defaults to the theme\u2019s link tokens.'
              : node.type === 'back_to_login_link'
                ? 'Renders as a link back to the realm\u2019s login page. Styling defaults to the theme\u2019s link tokens.'
                : 'Renders as a link to the realm\u2019s register page. Styling defaults to the theme\u2019s link tokens.'}
          </div>
          <ConfigSection title='Typography'>
            <SelectField
              label='Text align'
              value={(node.props.textAlign as string) || 'center'}
              options={[
                { label: 'Left', value: 'left' },
                { label: 'Center', value: 'center' },
                { label: 'Right', value: 'right' },
              ]}
              onChange={(v) => updateProp('textAlign', v)}
              allowEmpty={false}
            />
            <ColorField
              label='Color override'
              value={node.props.color as string}
              onChange={(v) => updateProp('color', v)}
            />
            <DimensionInput
              label='Font size override'
              value={node.props.fontSize as string}
              onChange={(v) => updateProp('fontSize', v)}
            />
            <TextField
              label='Font weight override'
              value={node.props.fontWeight as string}
              onChange={(v) => updateProp('fontWeight', v)}
              placeholder='400, 500, 600…'
            />
          </ConfigSection>
        </div>
      )

    case 'totp_qr_code':
      return (
        <div className='flex flex-col'>
          {identity}
          <div className='border-b border-border bg-muted/40 px-3 py-2 text-[11px] text-muted-foreground'>
            QR code rendered from the `otpauth://` URL the backend supplies during TOTP enrolment. Only the size and alignment are configurable; the content comes from the auth flow.
          </div>
          <ConfigSection title='Layout'>
            <DimensionInput
              label='Size'
              value={(node.props.size as string) ?? '180'}
              onChange={(v) => updateProp('size', v)}
            />
            <SelectField
              label='Alignment'
              value={(node.props.align as string) || 'center'}
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

    case 'totp_secret':
      return (
        <div className='flex flex-col'>
          {identity}
          <div className='border-b border-border bg-muted/40 px-3 py-2 text-[11px] text-muted-foreground'>
            Plain-text TOTP secret with a copy-on-click affordance. The value comes from the auth flow; only the alignment is configurable here.
          </div>
          <ConfigSection title='Layout'>
            <SelectField
              label='Alignment'
              value={(node.props.align as string) || 'center'}
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

    case 'form_error_banner':
      return (
        <div className='flex flex-col'>
          {identity}
          <div className='border-b border-border bg-muted/40 px-3 py-2 text-[11px] text-muted-foreground'>
            Auto-populated by the page&apos;s submit handler with the latest failure message (invalid credentials, etc.). Hides itself when there&apos;s nothing to show.
          </div>
          <ConfigSection title='Style'>
            <SelectField
              label='Variant'
              value={(node.props.variant as string) || 'destructive'}
              options={[
                { label: 'Destructive (red)', value: 'destructive' },
                { label: 'Warning (amber)', value: 'warning' },
                { label: 'Info (blue)', value: 'info' },
              ]}
              onChange={(v) => updateProp('variant', v)}
              allowEmpty={false}
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
