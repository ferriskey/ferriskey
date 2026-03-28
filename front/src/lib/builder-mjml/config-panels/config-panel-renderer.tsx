import type { ReactNode } from 'react'
import type { BuilderNode } from '../../builder-core'
import {
  TextField,
  ColorField,
  SelectField,
  PaddingFields,
} from './shared-fields'
import { RichTextEditor } from './rich-text-editor'

type OnUpdate = (
  updates: Partial<Pick<BuilderNode, 'props' | 'styles' | 'content'>>,
) => void

interface TemplateVariable {
  name: string
  description: string
}

export function renderMjmlConfigPanel(
  node: BuilderNode,
  onUpdate: OnUpdate,
  variables?: TemplateVariable[],
): ReactNode {
  const updateProp = (key: string, value: string) => {
    onUpdate({ props: { [key]: value } })
  }

  switch (node.type) {
    case 'mj-section':
      return (
        <div className='flex flex-col gap-3'>
          <ColorField
            label='Background'
            value={node.props['background-color'] as string}
            onChange={(v) => updateProp('background-color', v)}
          />
          <TextField
            label='Border'
            value={node.props['border'] as string}
            onChange={(v) => updateProp('border', v)}
          />
          <TextField
            label='Border Radius'
            value={node.props['border-radius'] as string}
            onChange={(v) => updateProp('border-radius', v)}
          />
          <SelectField
            label='Direction'
            value={node.props['direction'] as string}
            options={[
              { label: 'LTR', value: 'ltr' },
              { label: 'RTL', value: 'rtl' },
            ]}
            onChange={(v) => updateProp('direction', v)}
          />
          <SelectField
            label='Full Width'
            value={node.props['full-width'] as string}
            options={[{ label: 'Full Width', value: 'full-width' }]}
            onChange={(v) => updateProp('full-width', v)}
          />
          <PaddingFields node={node} onUpdate={onUpdate} />
        </div>
      )

    case 'mj-column':
      return (
        <div className='flex flex-col gap-3'>
          <TextField
            label='Width'
            value={node.props['width'] as string}
            onChange={(v) => updateProp('width', v)}
          />
          <ColorField
            label='Background'
            value={node.props['background-color'] as string}
            onChange={(v) => updateProp('background-color', v)}
          />
          <SelectField
            label='Vertical Align'
            value={node.props['vertical-align'] as string}
            options={[
              { label: 'Top', value: 'top' },
              { label: 'Middle', value: 'middle' },
              { label: 'Bottom', value: 'bottom' },
            ]}
            onChange={(v) => updateProp('vertical-align', v)}
          />
          <TextField
            label='Border'
            value={node.props['border'] as string}
            onChange={(v) => updateProp('border', v)}
          />
          <TextField
            label='Border Radius'
            value={node.props['border-radius'] as string}
            onChange={(v) => updateProp('border-radius', v)}
          />
          <PaddingFields node={node} onUpdate={onUpdate} />
        </div>
      )

    case 'mj-text':
      return (
        <div className='flex flex-col gap-3'>
          <span className='text-xs text-muted-foreground'>Content</span>
          <RichTextEditor
            content={node.content ?? ''}
            onChange={(html) => onUpdate({ content: html })}
            variables={variables}
          />
          <ColorField
            label='Color'
            value={node.props['color'] as string}
            onChange={(v) => updateProp('color', v)}
          />
          <TextField
            label='Font Family'
            value={node.props['font-family'] as string}
            onChange={(v) => updateProp('font-family', v)}
          />
          <TextField
            label='Font Size'
            value={node.props['font-size'] as string}
            onChange={(v) => updateProp('font-size', v)}
          />
          <SelectField
            label='Font Weight'
            value={node.props['font-weight'] as string}
            options={[
              { label: 'Normal', value: 'normal' },
              { label: 'Bold', value: 'bold' },
              { label: '300', value: '300' },
              { label: '400', value: '400' },
              { label: '600', value: '600' },
              { label: '700', value: '700' },
            ]}
            onChange={(v) => updateProp('font-weight', v)}
          />
          <TextField
            label='Line Height'
            value={node.props['line-height'] as string}
            onChange={(v) => updateProp('line-height', v)}
          />
          <SelectField
            label='Align'
            value={node.props['align'] as string}
            options={[
              { label: 'Left', value: 'left' },
              { label: 'Center', value: 'center' },
              { label: 'Right', value: 'right' },
              { label: 'Justify', value: 'justify' },
            ]}
            onChange={(v) => updateProp('align', v)}
          />
          <PaddingFields node={node} onUpdate={onUpdate} />
        </div>
      )

    case 'mj-image':
      return (
        <div className='flex flex-col gap-3'>
          <TextField
            label='Image URL'
            value={node.props['src'] as string}
            onChange={(v) => updateProp('src', v)}
          />
          <TextField
            label='Alt Text'
            value={node.props['alt'] as string}
            onChange={(v) => updateProp('alt', v)}
          />
          <TextField
            label='Link URL'
            value={node.props['href'] as string}
            onChange={(v) => updateProp('href', v)}
          />
          <TextField
            label='Width'
            value={node.props['width'] as string}
            onChange={(v) => updateProp('width', v)}
          />
          <SelectField
            label='Align'
            value={node.props['align'] as string}
            options={[
              { label: 'Left', value: 'left' },
              { label: 'Center', value: 'center' },
              { label: 'Right', value: 'right' },
            ]}
            onChange={(v) => updateProp('align', v)}
          />
          <TextField
            label='Border Radius'
            value={node.props['border-radius'] as string}
            onChange={(v) => updateProp('border-radius', v)}
          />
          <PaddingFields node={node} onUpdate={onUpdate} />
        </div>
      )

    case 'mj-button':
      return (
        <div className='flex flex-col gap-3'>
          <span className='text-xs text-muted-foreground'>Label</span>
          <RichTextEditor
            content={node.content ?? ''}
            onChange={(html) => onUpdate({ content: html })}
            variables={variables}
          />
          <TextField
            label='Link URL'
            value={node.props['href'] as string}
            onChange={(v) => updateProp('href', v)}
          />
          <ColorField
            label='Background'
            value={node.props['background-color'] as string}
            onChange={(v) => updateProp('background-color', v)}
          />
          <ColorField
            label='Text Color'
            value={node.props['color'] as string}
            onChange={(v) => updateProp('color', v)}
          />
          <TextField
            label='Font Size'
            value={node.props['font-size'] as string}
            onChange={(v) => updateProp('font-size', v)}
          />
          <TextField
            label='Border Radius'
            value={node.props['border-radius'] as string}
            onChange={(v) => updateProp('border-radius', v)}
          />
          <TextField
            label='Width'
            value={node.props['width'] as string}
            onChange={(v) => updateProp('width', v)}
          />
          <SelectField
            label='Align'
            value={node.props['align'] as string}
            options={[
              { label: 'Left', value: 'left' },
              { label: 'Center', value: 'center' },
              { label: 'Right', value: 'right' },
            ]}
            onChange={(v) => updateProp('align', v)}
          />
          <PaddingFields node={node} onUpdate={onUpdate} />
        </div>
      )

    case 'mj-divider':
      return (
        <div className='flex flex-col gap-3'>
          <ColorField
            label='Border Color'
            value={node.props['border-color'] as string}
            onChange={(v) => updateProp('border-color', v)}
          />
          <SelectField
            label='Border Style'
            value={node.props['border-style'] as string}
            options={[
              { label: 'Solid', value: 'solid' },
              { label: 'Dashed', value: 'dashed' },
              { label: 'Dotted', value: 'dotted' },
            ]}
            onChange={(v) => updateProp('border-style', v)}
          />
          <TextField
            label='Border Width'
            value={node.props['border-width'] as string}
            onChange={(v) => updateProp('border-width', v)}
          />
          <TextField
            label='Width'
            value={node.props['width'] as string}
            onChange={(v) => updateProp('width', v)}
          />
          <PaddingFields node={node} onUpdate={onUpdate} />
        </div>
      )

    case 'mj-spacer':
      return (
        <div className='flex flex-col gap-3'>
          <TextField
            label='Height'
            value={node.props['height'] as string}
            onChange={(v) => updateProp('height', v)}
          />
        </div>
      )

    case 'mj-hero':
      return (
        <div className='flex flex-col gap-3'>
          <ColorField
            label='Background'
            value={node.props['background-color'] as string}
            onChange={(v) => updateProp('background-color', v)}
          />
          <TextField
            label='Background URL'
            value={node.props['background-url'] as string}
            onChange={(v) => updateProp('background-url', v)}
          />
          <TextField
            label='Background Width'
            value={node.props['background-width'] as string}
            onChange={(v) => updateProp('background-width', v)}
          />
          <TextField
            label='Background Height'
            value={node.props['background-height'] as string}
            onChange={(v) => updateProp('background-height', v)}
          />
          <SelectField
            label='Mode'
            value={node.props['mode'] as string}
            options={[
              { label: 'Fixed Height', value: 'fixed-height' },
              { label: 'Fluid Height', value: 'fluid-height' },
            ]}
            onChange={(v) => updateProp('mode', v)}
          />
          <SelectField
            label='Vertical Align'
            value={node.props['vertical-align'] as string}
            options={[
              { label: 'Top', value: 'top' },
              { label: 'Middle', value: 'middle' },
              { label: 'Bottom', value: 'bottom' },
            ]}
            onChange={(v) => updateProp('vertical-align', v)}
          />
          <PaddingFields node={node} onUpdate={onUpdate} />
        </div>
      )

    case 'mj-wrapper':
      return (
        <div className='flex flex-col gap-3'>
          <ColorField
            label='Background'
            value={node.props['background-color'] as string}
            onChange={(v) => updateProp('background-color', v)}
          />
          <TextField
            label='Border'
            value={node.props['border'] as string}
            onChange={(v) => updateProp('border', v)}
          />
          <TextField
            label='Border Radius'
            value={node.props['border-radius'] as string}
            onChange={(v) => updateProp('border-radius', v)}
          />
          <SelectField
            label='Full Width'
            value={node.props['full-width'] as string}
            options={[{ label: 'Full Width', value: 'full-width' }]}
            onChange={(v) => updateProp('full-width', v)}
          />
          <PaddingFields node={node} onUpdate={onUpdate} />
        </div>
      )

    case 'mj-social':
      return (
        <div className='flex flex-col gap-3'>
          <SelectField
            label='Mode'
            value={node.props['mode'] as string}
            options={[
              { label: 'Horizontal', value: 'horizontal' },
              { label: 'Vertical', value: 'vertical' },
            ]}
            onChange={(v) => updateProp('mode', v)}
          />
          <SelectField
            label='Align'
            value={node.props['align'] as string}
            options={[
              { label: 'Left', value: 'left' },
              { label: 'Center', value: 'center' },
              { label: 'Right', value: 'right' },
            ]}
            onChange={(v) => updateProp('align', v)}
          />
          <TextField
            label='Icon Size'
            value={node.props['icon-size'] as string}
            onChange={(v) => updateProp('icon-size', v)}
          />
          <PaddingFields node={node} onUpdate={onUpdate} />
        </div>
      )

    case 'mj-navbar':
      return (
        <div className='flex flex-col gap-3'>
          <SelectField
            label='Align'
            value={node.props['align'] as string}
            options={[
              { label: 'Left', value: 'left' },
              { label: 'Center', value: 'center' },
              { label: 'Right', value: 'right' },
            ]}
            onChange={(v) => updateProp('align', v)}
          />
          <PaddingFields node={node} onUpdate={onUpdate} />
        </div>
      )

    case 'mj-table':
      return (
        <div className='flex flex-col gap-3'>
          <span className='text-xs text-muted-foreground'>
            Table HTML Content
          </span>
          <textarea
            className='min-h-[100px] rounded border border-border bg-background px-2 py-1 font-mono text-xs'
            value={node.content ?? ''}
            onChange={(e) => onUpdate({ content: e.target.value })}
          />
          <TextField
            label='Width'
            value={node.props['width'] as string}
            onChange={(v) => updateProp('width', v)}
          />
          <ColorField
            label='Color'
            value={node.props['color'] as string}
            onChange={(v) => updateProp('color', v)}
          />
          <TextField
            label='Font Size'
            value={node.props['font-size'] as string}
            onChange={(v) => updateProp('font-size', v)}
          />
          <PaddingFields node={node} onUpdate={onUpdate} />
        </div>
      )

    case 'mj-raw':
      return (
        <div className='flex flex-col gap-3'>
          <span className='text-xs text-muted-foreground'>
            Raw HTML Content
          </span>
          <textarea
            className='min-h-[100px] rounded border border-border bg-background px-2 py-1 font-mono text-xs'
            value={node.content ?? ''}
            onChange={(e) => onUpdate({ content: e.target.value })}
          />
        </div>
      )

    default:
      return (
        <div className='text-xs text-muted-foreground'>
          No configuration available for {node.type}
        </div>
      )
  }
}
