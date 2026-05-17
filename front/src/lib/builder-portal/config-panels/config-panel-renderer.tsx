import type { ReactNode } from 'react'
import type { BuilderNode } from '../../builder-core'
import { ConfigSection } from './config-section'
import { ColorField, SelectField, TextField } from './shared-fields'

type OnUpdate = (updates: Partial<Pick<BuilderNode, 'props' | 'styles' | 'content'>>) => void

export function renderPortalConfigPanel(node: BuilderNode, onUpdate: OnUpdate): ReactNode {
  const updateProp = (key: string, value: string) => onUpdate({ props: { [key]: value } })

  switch (node.type) {
    case 'container':
      return (
        <div className='flex flex-col'>
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

    case 'heading':
      return (
        <div className='flex flex-col'>
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

    case 'input':
      return (
        <div className='flex flex-col'>
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
        <div className='text-xs text-muted-foreground'>
          The page content slot is replaced at runtime by the page using this layout. No configuration needed.
        </div>
      )

    default:
      return (
        <div className='text-xs text-muted-foreground'>No configuration for {node.type}</div>
      )
  }
}
