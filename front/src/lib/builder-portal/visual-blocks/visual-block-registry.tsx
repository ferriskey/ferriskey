import type { CSSProperties, ReactNode } from 'react'
import { LayoutTemplate } from 'lucide-react'
import type { BuilderNode, ComponentDefinition } from '../../builder-core'
import { useBuilderContext } from '../../builder-core'
import {
  buttonStyle,
  containerStyle,
  divStyle,
  headingStyle,
  IdentityProvidersBlock,
  imageJustify,
  imageStyle,
  inputFieldStyle,
  inputHelperStyle,
  inputLabelStyle,
  orderStyle,
  resolveInputName,
  resolveInputType,
  textStyle,
} from '../renderer'
import { InlineTextEditor } from './inline-text-editor'

/**
 * Builder canvas chrome — we want the canvas to render byte-identical to the
 * runtime portal, so:
 *   - All visual styles come from renderer.tsx (single source of truth).
 *   - Selection / hover are signalled via `outline`, which does NOT take
 *     space in the layout (unlike borders or rings). The block's box model
 *     is therefore the same as at runtime.
 *   - Block-kind labels float as absolute overlays and only appear on hover
 *     or when the block is selected.
 */

const SELECTED_OUTLINE = '2px solid var(--fk-canvas-selected, #635dff)'

function chromeStyle(isSelected: boolean): CSSProperties {
  return {
    outline: isSelected ? SELECTED_OUTLINE : undefined,
    outlineOffset: isSelected ? 2 : 0,
  }
}

function mergeStyles(base: CSSProperties, isSelected: boolean): CSSProperties {
  return { ...base, ...chromeStyle(isSelected) }
}

/**
 * Floating "Container" / "Flex" / ... label shown on hover / selection.
 * Positioned absolutely so it doesn't push children around.
 */
function BlockLabel({
  label,
  visible,
}: {
  label: string
  visible: boolean
}) {
  return (
    <span
      style={{
        position: 'absolute',
        top: 4,
        left: 4,
        padding: '1px 4px',
        borderRadius: 3,
        backgroundColor: 'rgba(99, 93, 255, 0.85)',
        color: '#fff',
        fontSize: 10,
        fontWeight: 500,
        lineHeight: '14px',
        letterSpacing: 0.2,
        opacity: visible ? 1 : 0,
        pointerEvents: 'none',
        transition: 'opacity 120ms ease',
        zIndex: 5,
      }}
    >
      {label}
    </span>
  )
}

export function renderVisualBlock(
  node: BuilderNode,
  isSelected: boolean,
  children: ReactNode | undefined,
  componentDef?: ComponentDefinition,
): ReactNode {
  switch (node.type) {
    case 'container':
      return (
        <BoxBlock
          label='Container'
          node={node}
          isSelected={isSelected}
          style={containerStyle(node)}
        >
          {children}
        </BoxBlock>
      )
    // Legacy `flex` / `grid` block types are aliased to Div so trees saved
    // before the consolidation keep rendering.
    case 'flex':
    case 'grid':
    case 'div':
      return (
        <BoxBlock label='Div' node={node} isSelected={isSelected} style={divStyle(node)}>
          {children}
        </BoxBlock>
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
    case 'submit_button':
    case 'magic_link_button':
    case 'passkey_button':
      return <ButtonBlock node={node} isSelected={isSelected} />
    case 'input':
    case 'email_input':
    case 'password_input':
    case 'totp_input':
      return <InputBlock node={node} isSelected={isSelected} />
    case 'identity_providers':
      return <IdentityProvidersPreview node={node} isSelected={isSelected} />
    case 'page-content':
      return <PageContentSlot node={node} isSelected={isSelected} />
    default:
      return (
        <div
          style={{
            ...chromeStyle(isSelected),
            border: '1px dashed var(--border, #d4d4d8)',
            borderRadius: 4,
            padding: 12,
            fontSize: 12,
            color: 'var(--muted-foreground, #71717a)',
          }}
        >
          {componentDef?.label ?? node.type}
        </div>
      )
  }
}

/**
 * Container-like block (Container / Flex / Grid / Div). Renders the same
 * outer element as runtime, with an absolute label overlay and outline.
 */
function BoxBlock({
  label,
  node,
  isSelected,
  style,
  children,
}: {
  label: string
  node: BuilderNode
  isSelected: boolean
  style: CSSProperties
  children: ReactNode | undefined
}) {
  return (
    <div
      key={node.id}
      data-fk-id={node.id}
      // `position: relative` lets the floating label anchor here. If the
      // node's own `position` prop is set (e.g. div with `fixed`), it
      // overrides this via the spread below.
      style={{ position: 'relative', ...mergeStyles(style, isSelected) }}
      className='group/box'
    >
      <BlockLabel label={node.name?.trim() || label} visible={isSelected} />
      <span
        style={{ position: 'absolute', inset: 0, pointerEvents: 'none' }}
        className='hidden group-hover/box:block'
      />
      {children}
    </div>
  )
}

function EditableHeading({ node, isSelected }: { node: BuilderNode; isSelected: boolean }) {
  const { updateNode } = useBuilderContext()
  const level = (node.props.level as string) ?? '2'
  const Tag = `h${level}` as 'h1' | 'h2' | 'h3' | 'h4'
  const style = mergeStyles(headingStyle(node), isSelected)

  if (isSelected) {
    return (
      <Tag
        data-fk-id={node.id}
        style={style}
        onPointerDown={(e) => e.stopPropagation()}
        onClick={(e) => e.stopPropagation()}
      >
        <InlineTextEditor
          content={node.content ?? ''}
          onChange={(value) => updateNode(node.id, { content: value })}
        />
      </Tag>
    )
  }
  return <Tag data-fk-id={node.id} style={style}>{node.content ?? ''}</Tag>
}

function EditableText({ node, isSelected }: { node: BuilderNode; isSelected: boolean }) {
  const { updateNode } = useBuilderContext()
  const style = mergeStyles(textStyle(node), isSelected)

  if (isSelected) {
    return (
      <p
        data-fk-id={node.id}
        style={style}
        onPointerDown={(e) => e.stopPropagation()}
        onClick={(e) => e.stopPropagation()}
      >
        <InlineTextEditor
          content={node.content ?? ''}
          onChange={(value) => updateNode(node.id, { content: value })}
        />
      </p>
    )
  }
  return <p data-fk-id={node.id} style={style}>{node.content ?? ''}</p>
}

function ImageBlock({ node, isSelected }: { node: BuilderNode; isSelected: boolean }) {
  return (
    <div
      data-fk-id={node.id}
      style={{ display: 'flex', justifyContent: imageJustify(node), ...orderStyle(node) }}
    >
      <img
        src={(node.props.src as string) || ''}
        alt={(node.props.alt as string) || ''}
        style={mergeStyles(imageStyle(node), isSelected)}
      />
    </div>
  )
}

function SpacerBlock({ node, isSelected }: { node: BuilderNode; isSelected: boolean }) {
  const height = (node.props.height as string) || '16px'
  const width = (node.props.width as string) || '100%'
  return (
    <div
      data-fk-id={node.id}
      style={{
        ...chromeStyle(isSelected),
        height,
        width,
        position: 'relative',
        ...orderStyle(node),
      }}
    >
      {isSelected && (
        <span
          style={{
            position: 'absolute',
            inset: 0,
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            fontSize: 10,
            textTransform: 'uppercase',
            letterSpacing: 1,
            color: 'rgba(99, 93, 255, 0.55)',
            pointerEvents: 'none',
          }}
        >
          spacer
        </span>
      )}
    </div>
  )
}

function DividerBlock({ node, isSelected }: { node: BuilderNode; isSelected: boolean }) {
  return (
    <div
      data-fk-id={node.id}
      style={{ ...chromeStyle(isSelected), display: 'flex', justifyContent: 'center', ...orderStyle(node) }}
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
  const style = mergeStyles(buttonStyle(node), isSelected)

  // Render exactly as runtime (`<a>` for button). We only block the anchor's
  // default navigation — propagation must continue so the SortableNode
  // wrapper's onClick handler still picks the click up and selects the node.
  return (
    <a
      data-fk-id={node.id}
      href='#'
      style={style}
      onClick={(e) => {
        e.preventDefault()
      }}
    >
      {isSelected ? (
        <InlineTextEditor
          content={node.content ?? 'Button'}
          onChange={(value) => updateNode(node.id, { content: value })}
        />
      ) : (
        (node.content ?? 'Button')
      )}
    </a>
  )
}

function InputBlock({ node, isSelected }: { node: BuilderNode; isSelected: boolean }) {
  const label = (node.props.label as string) ?? ''
  const placeholder = (node.props.placeholder as string) ?? ''
  const helperText = (node.props.helperText as string) ?? ''

  return (
    <div
      data-fk-id={node.id}
      style={{ display: 'flex', flexDirection: 'column', gap: 6, ...orderStyle(node) }}
    >
      {label ? <label style={inputLabelStyle()}>{label}</label> : null}
      <input
        type={resolveInputType(node)}
        name={resolveInputName(node)}
        placeholder={placeholder}
        // Disabled in the canvas so the user can't type into the preview
        // by accident; the runtime renderer flips this off (`runtime: true`).
        disabled
        style={mergeStyles({ ...inputFieldStyle(), pointerEvents: 'none' }, isSelected)}
      />
      {helperText ? <span style={inputHelperStyle()}>{helperText}</span> : null}
    </div>
  )
}

function IdentityProvidersPreview({
  node,
  isSelected,
}: {
  node: BuilderNode
  isSelected: boolean
}) {
  return (
    <div style={chromeStyle(isSelected)}>
      {/* Canvas preview reuses the runtime renderer with `runtime=false` so
          the placeholder providers (Google / GitHub) are drawn — keeps the
          builder visually identical to what realm end-users will see. */}
      <IdentityProvidersBlock node={node} runtime={false} />
    </div>
  )
}

function PageContentSlot({ isSelected }: { node: BuilderNode; isSelected: boolean }) {
  return (
    <div
      style={{
        ...chromeStyle(isSelected),
        // Plain block-flow placeholder. Layout (centering, sizing) is up to
        // the slot's parent in the user's tree — matching the runtime
        // `<div data-portal-page-content>` exactly.
        minHeight: 120,
        boxSizing: 'border-box',
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
        gap: 8,
        padding: 24,
        borderRadius: 4,
        border: `2px dashed ${isSelected ? 'rgba(99,93,255,0.7)' : 'var(--border, #d4d4d8)'}`,
        backgroundColor: 'rgba(244, 244, 245, 0.5)',
      }}
    >
      <LayoutTemplate size={20} color='rgba(0,0,0,0.4)' />
      <span style={{ fontSize: 12, fontWeight: 500, color: 'rgba(0,0,0,0.55)' }}>
        Page content slot
      </span>
      <span style={{ fontSize: 10, color: 'rgba(0,0,0,0.45)' }}>
        Pages using this layout will render here
      </span>
    </div>
  )
}
