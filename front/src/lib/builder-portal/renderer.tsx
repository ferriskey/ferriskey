import type { CSSProperties, ReactNode } from 'react'
import type { BuilderNode } from '../builder-core'

type RenderOptions = {
  /** Replaces <page-content /> nodes with this slot content (used by layouts). */
  pageContent?: ReactNode
}

export function treeToReactNode(tree: BuilderNode[], options: RenderOptions = {}): ReactNode {
  return tree.map((node) => renderNode(node, options))
}

function renderNode(node: BuilderNode, options: RenderOptions): ReactNode {
  switch (node.type) {
    case 'container':
      return (
        <div
          key={node.id}
          style={containerStyle(node)}
        >
          {node.children.length > 0
            ? node.children.map((c) => renderNode(c, options))
            : null}
        </div>
      )

    case 'heading': {
      const level = (node.props.level as string) ?? '2'
      const Tag = (`h${level}` as 'h1' | 'h2' | 'h3' | 'h4')
      return (
        <Tag key={node.id} style={headingStyle(node)}>
          {node.content ?? ''}
        </Tag>
      )
    }

    case 'text':
      return (
        <p key={node.id} style={textStyle(node)}>
          {node.content ?? ''}
        </p>
      )

    case 'image':
      return (
        <div
          key={node.id}
          style={{ display: 'flex', justifyContent: imageJustify(node) }}
        >
          <img
            src={(node.props.src as string) || ''}
            alt={(node.props.alt as string) || ''}
            style={imageStyle(node)}
          />
        </div>
      )

    case 'spacer':
      return (
        <div
          key={node.id}
          style={{ height: (node.props.height as string) || '16px' }}
        />
      )

    case 'divider':
      return (
        <hr
          key={node.id}
          style={{
            border: 'none',
            borderTop: `${(node.props.thickness as string) || '1px'} solid ${
              (node.props.color as string) || '#e5e7eb'
            }`,
            width: (node.props.width as string) || '100%',
            margin: 0,
          }}
        />
      )

    case 'button':
      return (
        <a key={node.id} href={(node.props.href as string) || '#'} style={buttonStyle(node)}>
          {node.content ?? 'Button'}
        </a>
      )

    case 'input':
      return (
        <div key={node.id} style={{ display: 'flex', flexDirection: 'column', gap: 6 }}>
          {node.props.label ? (
            <label style={inputLabelStyle()}>{node.props.label as string}</label>
          ) : null}
          <input
            type={(node.props.type as string) || 'text'}
            placeholder={(node.props.placeholder as string) || ''}
            style={inputFieldStyle()}
            disabled
          />
          {node.props.helperText ? (
            <span style={inputHelperStyle()}>{node.props.helperText as string}</span>
          ) : null}
        </div>
      )

    case 'page-content':
      return (
        <div key={node.id} data-portal-page-content>
          {options.pageContent ?? null}
        </div>
      )

    default:
      return null
  }
}

function containerStyle(node: BuilderNode): CSSProperties {
  return {
    display: 'flex',
    flexDirection: ((node.props.direction as string) || 'column') as 'row' | 'column',
    alignItems: (node.props.align as string) || 'stretch',
    gap: (node.props.gap as string) || '12px',
    padding: (node.props.padding as string) || '16px',
    backgroundColor: (node.props.backgroundColor as string) || undefined,
    borderRadius: (node.props.borderRadius as string) || undefined,
    width: (node.props.width as string) || undefined,
  }
}

function headingStyle(node: BuilderNode): CSSProperties {
  return {
    color: (node.props.color as string) || 'var(--fk-color-body-text, #111827)',
    textAlign: ((node.props.textAlign as string) || 'center') as CSSProperties['textAlign'],
    fontSize: (node.props.fontSize as string) || undefined,
    fontWeight: (node.props.fontWeight as string) || '600',
    margin: 0,
  }
}

function textStyle(node: BuilderNode): CSSProperties {
  return {
    color: (node.props.color as string) || 'var(--fk-color-body-text, #374151)',
    textAlign: ((node.props.textAlign as string) || 'left') as CSSProperties['textAlign'],
    fontSize: (node.props.fontSize as string) || 'var(--fk-font-base-size, 16px)',
    fontWeight: (node.props.fontWeight as string) || undefined,
    lineHeight: (node.props.lineHeight as string) || '1.5',
    margin: 0,
  }
}

function imageStyle(node: BuilderNode): CSSProperties {
  return {
    width: (node.props.width as string) || undefined,
    height: (node.props.height as string) || undefined,
    borderRadius: (node.props.borderRadius as string) || undefined,
    maxWidth: '100%',
  }
}

function imageJustify(node: BuilderNode): CSSProperties['justifyContent'] {
  const align = (node.props.align as string) || 'center'
  if (align === 'left') return 'flex-start'
  if (align === 'right') return 'flex-end'
  return 'center'
}

function inputLabelStyle(): CSSProperties {
  return {
    fontSize: '13px',
    fontWeight: 500,
    color: 'var(--fk-color-body-text, #374151)',
  }
}

function inputFieldStyle(): CSSProperties {
  return {
    width: '100%',
    padding: '10px 12px',
    border: 'var(--fk-border-input, 1px) solid var(--fk-color-body-text, #d1d5db)',
    borderRadius: 'var(--fk-radius-input, 6px)',
    fontSize: 'var(--fk-font-base-size, 14px)',
    backgroundColor: '#fff',
    color: 'var(--fk-color-body-text, #111827)',
  }
}

function inputHelperStyle(): CSSProperties {
  return {
    fontSize: '12px',
    color: 'var(--fk-color-body-text, #6b7280)',
    textAlign: 'right',
  }
}

function buttonStyle(node: BuilderNode): CSSProperties {
  const variant = (node.props.variant as string) || 'primary'
  const fullWidth = (node.props.fullWidth as string) === 'true'

  const base: CSSProperties = {
    display: 'inline-flex',
    alignItems: 'center',
    justifyContent: 'center',
    padding: '10px 16px',
    borderRadius: 'var(--fk-radius-button, 6px)',
    fontWeight: 500,
    textDecoration: 'none',
    cursor: 'pointer',
    width: fullWidth ? '100%' : 'auto',
    border: 'var(--fk-border-button, 1px) solid transparent',
    transition: 'opacity 0.15s ease',
  }

  if (variant === 'secondary') {
    return {
      ...base,
      backgroundColor: 'var(--fk-color-secondary-button, #ffffff)',
      color: 'var(--fk-color-secondary-button-label, #111827)',
      borderColor: 'var(--fk-color-body-text, #d1d5db)',
    }
  }

  if (variant === 'outline') {
    return {
      ...base,
      backgroundColor: 'transparent',
      color: 'var(--fk-color-primary-button, #635dff)',
      borderColor: 'var(--fk-color-primary-button, #635dff)',
    }
  }

  return {
    ...base,
    backgroundColor: 'var(--fk-color-primary-button, #635dff)',
    color: 'var(--fk-color-primary-button-label, #ffffff)',
  }
}
