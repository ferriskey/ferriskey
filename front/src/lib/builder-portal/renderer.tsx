import type { CSSProperties, ReactNode } from 'react'
import type { BuilderNode } from '../builder-core'

type RenderOptions = {
  /** Replaces <page-content /> nodes with this slot content (used by layouts). */
  pageContent?: ReactNode
  /**
   * When true, the tree is rendered for the live portal (not the builder
   * preview), so inputs accept user input instead of being disabled.
   */
  runtime?: boolean
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

    // `flex` and `grid` are legacy block types. Trees saved before they were
    // collapsed into Div still reference them; both render through divStyle
    // which respects the `display` prop (forced to flex/grid here for back-
    // compat), so old data renders identically.
    case 'flex':
    case 'grid':
    case 'div':
      return (
        <div key={node.id} style={divStyle(node)}>
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

    case 'submit_button':
      // In the runtime portal, the submit button drives the host <form>'s
      // onSubmit handler. In the builder preview it stays a non-interactive
      // link so it visually matches the regular button without firing nav.
      return options.runtime ? (
        <button
          key={node.id}
          type='submit'
          style={{ ...buttonStyle(node), border: 'none', cursor: 'pointer' }}
        >
          {node.content ?? 'Submit'}
        </button>
      ) : (
        <a key={node.id} href='#' style={buttonStyle(node)}>
          {node.content ?? 'Submit'}
        </a>
      )

    case 'input':
    case 'email_input':
    case 'password_input':
    case 'totp_input':
      return (
        <div key={node.id} style={{ display: 'flex', flexDirection: 'column', gap: 6 }}>
          {node.props.label ? (
            <label style={inputLabelStyle()}>{node.props.label as string}</label>
          ) : null}
          <input
            type={resolveInputType(node)}
            name={resolveInputName(node)}
            placeholder={(node.props.placeholder as string) || ''}
            style={inputFieldStyle()}
            disabled={!options.runtime}
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

export function containerStyle(node: BuilderNode): CSSProperties {
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

/**
 * Resolves the effective `display` value for a Div-like node. Legacy `flex`
 * and `grid` block types coerce to those displays so trees saved before the
 * consolidation keep rendering correctly.
 */
function resolveDisplay(node: BuilderNode): string {
  if (node.type === 'flex') return 'flex'
  if (node.type === 'grid') return 'grid'
  return (node.props.display as string) || 'block'
}

export function divStyle(node: BuilderNode): CSSProperties {
  const z = (node.props.zIndex as string) || ''
  const display = resolveDisplay(node)

  const base: CSSProperties = {
    display: display as CSSProperties['display'],
    position: ((node.props.position as string) || 'static') as CSSProperties['position'],
    top: (node.props.top as string) || undefined,
    right: (node.props.right as string) || undefined,
    bottom: (node.props.bottom as string) || undefined,
    left: (node.props.left as string) || undefined,
    width: (node.props.width as string) || undefined,
    height: (node.props.height as string) || undefined,
    minWidth: (node.props.minWidth as string) || undefined,
    maxWidth: (node.props.maxWidth as string) || undefined,
    minHeight: (node.props.minHeight as string) || undefined,
    maxHeight: (node.props.maxHeight as string) || undefined,
    padding: (node.props.padding as string) || undefined,
    margin: (node.props.margin as string) || undefined,
    gap: (node.props.gap as string) || undefined,
    backgroundColor: (node.props.backgroundColor as string) || undefined,
    borderRadius: (node.props.borderRadius as string) || undefined,
    overflow: ((node.props.overflow as string) || 'visible') as CSSProperties['overflow'],
    zIndex: z ? Number(z) : undefined,
  }

  if (display === 'flex') {
    return {
      ...base,
      flexDirection: ((node.props.direction as string) || 'row') as CSSProperties['flexDirection'],
      flexWrap: ((node.props.wrap as string) || 'nowrap') as CSSProperties['flexWrap'],
      justifyContent: (node.props.justifyContent as string) || 'flex-start',
      alignItems: (node.props.alignItems as string) || 'stretch',
      alignContent: (node.props.alignContent as string) || 'stretch',
    }
  }

  if (display === 'grid') {
    return {
      ...base,
      gridTemplateColumns: (node.props.templateColumns as string) || undefined,
      gridTemplateRows: (node.props.templateRows as string) || undefined,
      columnGap: (node.props.columnGap as string) || undefined,
      rowGap: (node.props.rowGap as string) || undefined,
      justifyItems: (node.props.justifyItems as string) || undefined,
      alignItems: (node.props.alignItems as string) || undefined,
      gridAutoFlow: (node.props.autoFlow as string) || undefined,
    }
  }

  return base
}

export function headingStyle(node: BuilderNode): CSSProperties {
  return {
    color: (node.props.color as string) || 'var(--fk-color-body-text, #111827)',
    textAlign: ((node.props.textAlign as string) || 'center') as CSSProperties['textAlign'],
    fontSize: (node.props.fontSize as string) || undefined,
    fontWeight: (node.props.fontWeight as string) || '600',
    margin: 0,
  }
}

export function textStyle(node: BuilderNode): CSSProperties {
  return {
    color: (node.props.color as string) || 'var(--fk-color-body-text, #374151)',
    textAlign: ((node.props.textAlign as string) || 'left') as CSSProperties['textAlign'],
    fontSize: (node.props.fontSize as string) || 'var(--fk-font-base-size, 16px)',
    fontWeight: (node.props.fontWeight as string) || undefined,
    lineHeight: (node.props.lineHeight as string) || '1.5',
    margin: 0,
  }
}

export function imageStyle(node: BuilderNode): CSSProperties {
  return {
    width: (node.props.width as string) || undefined,
    height: (node.props.height as string) || undefined,
    borderRadius: (node.props.borderRadius as string) || undefined,
    maxWidth: '100%',
  }
}

export function imageJustify(node: BuilderNode): CSSProperties['justifyContent'] {
  const align = (node.props.align as string) || 'center'
  if (align === 'left') return 'flex-start'
  if (align === 'right') return 'flex-end'
  return 'center'
}

export function inputLabelStyle(): CSSProperties {
  return {
    fontSize: '13px',
    fontWeight: 500,
    color: 'var(--fk-color-body-text, #374151)',
  }
}

export function inputFieldStyle(): CSSProperties {
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

export function inputHelperStyle(): CSSProperties {
  return {
    fontSize: '12px',
    color: 'var(--fk-color-body-text, #6b7280)',
    textAlign: 'right',
  }
}

export function buttonStyle(node: BuilderNode): CSSProperties {
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

/**
 * Required-block inputs always render the HTML type that matches their
 * semantic role — admins can't relabel an `email_input` into a `text` field.
 * The generic `input` block stays user-controlled.
 */
export function resolveInputType(node: BuilderNode): string {
  switch (node.type) {
    // `email_input` is used as the identifier field on login flows where the
    // realm may accept a username, an email, or both — so we render `text`
    // and let the backend validate. Browser-level `type=email` would block
    // submission for plain usernames.
    case 'email_input':
      return 'text'
    case 'password_input':
      return 'password'
    case 'totp_input':
      return 'text'
    default:
      return (node.props.type as string) || 'text'
  }
}

/**
 * Required-block inputs also lock their `name` so the page submit handler
 * can find them. Generic `input` is free to be named anything.
 */
export function resolveInputName(node: BuilderNode): string | undefined {
  switch (node.type) {
    case 'email_input':
      return 'email'
    case 'password_input':
      return 'password'
    case 'totp_input':
      return 'totp'
    default:
      return (node.props.name as string) || undefined
  }
}
