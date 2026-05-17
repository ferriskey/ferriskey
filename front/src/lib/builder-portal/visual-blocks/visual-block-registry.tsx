import type { ReactNode } from 'react'
import { LayoutTemplate } from 'lucide-react'
import type { BuilderNode, ComponentDefinition } from '../../builder-core'
import { useBuilderContext } from '../../builder-core'
import { InlineTextEditor } from './inline-text-editor'
import { ContainerBlock } from './container-block'

export function renderVisualBlock(
  node: BuilderNode,
  isSelected: boolean,
  children: ReactNode | undefined,
  componentDef?: ComponentDefinition,
): ReactNode {
  switch (node.type) {
    case 'container':
      return (
        <ContainerBlock node={node} isSelected={isSelected}>
          {children}
        </ContainerBlock>
      )
    case 'heading':
      return <EditableHeading node={node} isSelected={isSelected} />
    case 'text':
      return <EditableText node={node} isSelected={isSelected} />
    case 'image':
      return <ImageBlock node={node} isSelected={isSelected} />
    case 'spacer':
      return <SpacerBlock node={node} isSelected={isSelected} />
    case 'divider':
      return <DividerBlock node={node} isSelected={isSelected} />
    case 'button':
      return <ButtonBlock node={node} isSelected={isSelected} />
    case 'input':
      return <InputBlock node={node} isSelected={isSelected} />
    case 'page-content':
      return <PageContentSlot node={node} isSelected={isSelected} />
    default:
      return (
        <div
          className={`flex items-center gap-1.5 rounded border border-dashed p-3 text-xs text-muted-foreground ${
            isSelected ? 'ring-2 ring-primary' : ''
          }`}
        >
          {componentDef?.icon}
          <span>{componentDef?.label ?? node.type}</span>
        </div>
      )
  }
}

function EditableHeading({ node, isSelected }: { node: BuilderNode; isSelected: boolean }) {
  const { updateNode } = useBuilderContext()
  const level = (node.props.level as string) ?? '2'
  const Tag = `h${level}` as 'h1' | 'h2' | 'h3' | 'h4'
  const style = {
    color: (node.props.color as string) || 'var(--fk-color-body-text, #111827)',
    textAlign: ((node.props.textAlign as string) || 'center') as 'left' | 'center' | 'right',
    fontWeight: (node.props.fontWeight as string) || '600',
    margin: 0,
  }

  if (isSelected) {
    return (
      <Tag style={style} onPointerDown={(e) => e.stopPropagation()} onClick={(e) => e.stopPropagation()}>
        <InlineTextEditor
          content={node.content ?? ''}
          onChange={(value) => updateNode(node.id, { content: value })}
        />
      </Tag>
    )
  }
  return <Tag style={style}>{node.content ?? ''}</Tag>
}

function EditableText({ node, isSelected }: { node: BuilderNode; isSelected: boolean }) {
  const { updateNode } = useBuilderContext()
  const style = {
    color: (node.props.color as string) || 'var(--fk-color-body-text, #374151)',
    textAlign: ((node.props.textAlign as string) || 'left') as 'left' | 'center' | 'right',
    fontSize: (node.props.fontSize as string) || 'var(--fk-font-base-size, 16px)',
    lineHeight: (node.props.lineHeight as string) || '1.5',
    margin: 0,
  }

  if (isSelected) {
    return (
      <p style={style} onPointerDown={(e) => e.stopPropagation()} onClick={(e) => e.stopPropagation()}>
        <InlineTextEditor
          content={node.content ?? ''}
          onChange={(value) => updateNode(node.id, { content: value })}
        />
      </p>
    )
  }
  return <p style={style}>{node.content ?? ''}</p>
}

function ImageBlock({ node, isSelected }: { node: BuilderNode; isSelected: boolean }) {
  const align = (node.props.align as string) || 'center'
  const justify = align === 'left' ? 'flex-start' : align === 'right' ? 'flex-end' : 'center'
  return (
    <div
      className={`flex w-full transition-all ${
        isSelected ? 'ring-2 ring-primary' : 'hover:ring-1 hover:ring-dashed hover:ring-border'
      }`}
      style={{ justifyContent: justify, padding: 4 }}
    >
      <img
        src={(node.props.src as string) || ''}
        alt={(node.props.alt as string) || ''}
        style={{
          width: (node.props.width as string) || undefined,
          height: (node.props.height as string) || undefined,
          borderRadius: (node.props.borderRadius as string) || undefined,
          maxWidth: '100%',
        }}
      />
    </div>
  )
}

function SpacerBlock({ node, isSelected }: { node: BuilderNode; isSelected: boolean }) {
  return (
    <div
      className={`relative w-full transition-all ${
        isSelected ? 'ring-2 ring-primary' : 'hover:ring-1 hover:ring-dashed hover:ring-border'
      }`}
      style={{ height: (node.props.height as string) || '16px' }}
    >
      <span className='absolute inset-0 flex items-center justify-center text-[10px] uppercase tracking-wide text-muted-foreground/60'>
        spacer
      </span>
    </div>
  )
}

function DividerBlock({ node, isSelected }: { node: BuilderNode; isSelected: boolean }) {
  return (
    <div
      className={`flex w-full justify-center transition-all ${
        isSelected ? 'ring-2 ring-primary' : 'hover:ring-1 hover:ring-dashed hover:ring-border'
      }`}
      style={{ padding: 4 }}
    >
      <hr
        style={{
          width: (node.props.width as string) || '100%',
          border: 'none',
          borderTop: `${(node.props.thickness as string) || '1px'} solid ${
            (node.props.color as string) || '#e5e7eb'
          }`,
          margin: 0,
        }}
      />
    </div>
  )
}

function ButtonBlock({ node, isSelected }: { node: BuilderNode; isSelected: boolean }) {
  const { updateNode } = useBuilderContext()
  const variant = (node.props.variant as string) || 'primary'
  const fullWidth = (node.props.fullWidth as string) === 'true'

  const colors =
    variant === 'secondary'
      ? {
          backgroundColor: 'var(--fk-color-secondary-button, #ffffff)',
          color: 'var(--fk-color-secondary-button-label, #111827)',
          border: '1px solid var(--fk-color-body-text, #d1d5db)',
        }
      : variant === 'outline'
        ? {
            backgroundColor: 'transparent',
            color: 'var(--fk-color-primary-button, #635dff)',
            border: '1px solid var(--fk-color-primary-button, #635dff)',
          }
        : {
            backgroundColor: 'var(--fk-color-primary-button, #635dff)',
            color: 'var(--fk-color-primary-button-label, #ffffff)',
            border: '1px solid transparent',
          }

  return (
    <div className={`transition-all ${isSelected ? 'ring-2 ring-primary' : 'hover:ring-1 hover:ring-dashed hover:ring-border'}`}>
      <div
        style={{
          ...colors,
          display: 'inline-flex',
          alignItems: 'center',
          justifyContent: 'center',
          padding: '10px 16px',
          borderRadius: 'var(--fk-radius-button, 6px)',
          fontWeight: 500,
          width: fullWidth ? '100%' : 'auto',
        }}
      >
        {isSelected ? (
          <InlineTextEditor
            content={node.content ?? 'Button'}
            onChange={(value) => updateNode(node.id, { content: value })}
          />
        ) : (
          node.content ?? 'Button'
        )}
      </div>
    </div>
  )
}

function InputBlock({ node, isSelected }: { node: BuilderNode; isSelected: boolean }) {
  const label = (node.props.label as string) ?? ''
  const placeholder = (node.props.placeholder as string) ?? ''
  const type = (node.props.type as string) ?? 'text'
  const helperText = (node.props.helperText as string) ?? ''

  return (
    <div
      className={`flex w-full flex-col gap-1.5 rounded transition-all ${
        isSelected ? 'ring-2 ring-primary' : 'hover:ring-1 hover:ring-dashed hover:ring-border'
      }`}
      style={{ padding: 2 }}
    >
      {label ? (
        <label
          style={{
            fontSize: '13px',
            fontWeight: 500,
            color: 'var(--fk-color-body-text, #374151)',
          }}
        >
          {label}
        </label>
      ) : null}
      <input
        type={type}
        placeholder={placeholder}
        disabled
        style={{
          width: '100%',
          padding: '10px 12px',
          border: 'var(--fk-border-input, 1px) solid var(--fk-color-body-text, #d1d5db)',
          borderRadius: 'var(--fk-radius-input, 6px)',
          fontSize: 'var(--fk-font-base-size, 14px)',
          backgroundColor: '#fff',
          color: 'var(--fk-color-body-text, #111827)',
          pointerEvents: 'none',
        }}
      />
      {helperText ? (
        <span
          style={{
            fontSize: '12px',
            color: 'var(--fk-color-body-text, #6b7280)',
            textAlign: 'right',
          }}
        >
          {helperText}
        </span>
      ) : null}
    </div>
  )
}

function PageContentSlot({ isSelected }: { node: BuilderNode; isSelected: boolean }) {
  return (
    <div
      className={`flex min-h-[120px] flex-col items-center justify-center gap-2 rounded border-2 border-dashed bg-muted/40 p-6 transition-all ${
        isSelected ? 'border-primary' : 'border-border'
      }`}
    >
      <LayoutTemplate size={20} className='text-muted-foreground' />
      <span className='text-xs font-medium text-muted-foreground'>Page content slot</span>
      <span className='text-[10px] text-muted-foreground/70'>
        Pages using this layout will render here
      </span>
    </div>
  )
}
