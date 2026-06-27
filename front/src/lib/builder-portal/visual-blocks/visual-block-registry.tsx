import type { CSSProperties, ReactNode } from 'react'
import { Ban, KeyRound, LayoutTemplate, Mail } from 'lucide-react'
import type { BuilderNode, ComponentDefinition } from '../../builder-core'
import { useBuilderContext } from '../../builder-core'
import { PortalInput } from '../components/portal-input'
import {
  buttonStyle,
  cardContentStyle,
  cardFooterStyle,
  cardHeaderStyle,
  cardStyle,
  containerStyle,
  divStyle,
  forgotPasswordLinkStyle,
  headingStyle,
  IdentityProvidersBlock,
  imageJustify,
  imageStyle,
  inputHelperStyle,
  inputLabelStyle,
  magicLinkButtonStyle,
  orderStyle,
  passkeyButtonStyle,
  resolveAutoComplete,
  resolveInputName,
  resolveInputType,
  textStyle,
  TotpInputField,
  UserCodeInputField,
  TotpQrCodeBlock,
  TotpSecretBlock,
  FormErrorBannerBlock,
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
    case 'card':
      return (
        <BoxBlock label='Card' node={node} isSelected={isSelected} style={cardStyle(node)}>
          {children}
        </BoxBlock>
      )
    case 'card-header':
      return (
        <BoxBlock label='Header' node={node} isSelected={isSelected} style={cardHeaderStyle(node)}>
          {children}
        </BoxBlock>
      )
    case 'card-content':
      return (
        <BoxBlock label='Content' node={node} isSelected={isSelected} style={cardContentStyle(node)}>
          {children}
        </BoxBlock>
      )
    case 'card-footer':
      return (
        <BoxBlock label='Footer' node={node} isSelected={isSelected} style={cardFooterStyle(node)}>
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
    case 'device_approve_button':
    case 'device_deny_button':
    case 'magic_link_button':
    case 'passkey_button':
      return <ButtonBlock node={node} isSelected={isSelected} />
    case 'input':
    case 'username_input':
    case 'first_name_input':
    case 'last_name_input':
    case 'email_input':
    case 'password_input':
    case 'password_confirm_input':
      return <InputBlock node={node} isSelected={isSelected} />
    case 'totp_input':
      return <TotpInputBlock node={node} isSelected={isSelected} />
    case 'user_code_input':
      return <UserCodeInputBlock node={node} isSelected={isSelected} />
    case 'identity_providers':
      return <IdentityProvidersPreview node={node} isSelected={isSelected} />
    case 'forgot_password_link':
      return <EditableForgotPasswordLink node={node} isSelected={isSelected} />
    case 'back_to_login_link':
      return <EditableBackToLoginLink node={node} isSelected={isSelected} />
    case 'register_link':
      return <EditableRegisterLink node={node} isSelected={isSelected} />
    case 'totp_qr_code':
      // Canvas preview: reuse the runtime block with no `totpSetup`
      // option — it renders the placeholder QR square + a selection
      // outline so the admin can position it before any real data flows.
      return (
        <div style={chromeStyle(isSelected)}>
          <TotpQrCodeBlock node={node} options={{}} />
        </div>
      )
    case 'totp_secret':
      return (
        <div style={chromeStyle(isSelected)}>
          <TotpSecretBlock node={node} options={{}} />
        </div>
      )
    case 'form_error_banner':
      // Pass no `formError`/`runtime` so the renderer falls back to the
      // placeholder sample message and the admin can style/position
      // the banner without having to trigger a real failure.
      return (
        <div style={chromeStyle(isSelected)}>
          <FormErrorBannerBlock node={node} options={{}} />
        </div>
      )
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
  // Pick the right style fn per type so the canvas mirrors the runtime
  // (`magic_link_button` and `passkey_button` have their own theme tokens —
  // they don't follow the generic Primary/Secondary palette anymore).
  const styleFn =
    node.type === 'magic_link_button'
      ? magicLinkButtonStyle
      : node.type === 'passkey_button'
        ? passkeyButtonStyle
        : buttonStyle
  // Icon prefix: alternative-auth buttons get a glyph (mail / key) so the
  // canvas matches the runtime exactly. Generic buttons stay text-only.
  const icon =
    node.type === 'magic_link_button' ? (
      <Mail size={16} aria-hidden />
    ) : node.type === 'passkey_button' ? (
      <KeyRound size={16} aria-hidden />
    ) : node.type === 'device_deny_button' ? (
      <Ban size={16} aria-hidden />
    ) : null
  const style = mergeStyles(
    {
      ...styleFn(node),
      gap: icon ? 8 : undefined,
    },
    isSelected,
  )

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
      {icon}
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
      style={{
        ...chromeStyle(isSelected),
        display: 'flex',
        flexDirection: 'column',
        gap: 6,
        ...orderStyle(node),
      }}
    >
      {/* Reuse the runtime `PortalInput` with `disabled` so the canvas
          preview is byte-identical to what realm users will see — same
          floating-label visual, same theme tokens, same focus ring. */}
      <PortalInput
        name={resolveInputName(node)}
        label={label}
        type={resolveInputType(node)}
        placeholder={placeholder}
        disabled
        autoComplete={resolveAutoComplete(node)}
        alwaysFloatLabel
      />
      {helperText ? <span style={inputHelperStyle()}>{helperText}</span> : null}
    </div>
  )
}

/**
 * Canvas preview of the segmented OTP input. Reuses the runtime
 * `TotpInputField` (with `disabled=true`) so the slots render byte-
 * identical to what realm end-users will see — no risk of the admin
 * shipping a layout that looks different from the preview.
 */
function TotpInputBlock({ node, isSelected }: { node: BuilderNode; isSelected: boolean }) {
  const label = (node.props.label as string) ?? ''
  const helperText = (node.props.helperText as string) ?? ''
  const rawLength = (node.props.length as string) || '6'
  const parsed = Number.parseInt(rawLength, 10)
  const length = Number.isFinite(parsed) && parsed > 0 && parsed <= 12 ? parsed : 6

  return (
    <div
      data-fk-id={node.id}
      style={{
        ...chromeStyle(isSelected),
        display: 'flex',
        flexDirection: 'column',
        gap: 6,
        alignItems: 'center',
        // Disable pointer events on the whole preview — same protection as
        // the regular `InputBlock` so the admin can't accidentally focus a
        // slot while editing the page.
        pointerEvents: 'none',
        ...orderStyle(node),
      }}
    >
      {label ? <label style={inputLabelStyle()}>{label}</label> : null}
      <TotpInputField disabled name={resolveInputName(node) ?? 'totp'} length={length} />
      {helperText ? <span style={inputHelperStyle()}>{helperText}</span> : null}
    </div>
  )
}

/**
 * Canvas preview of the device user-code input. Mirrors `TotpInputBlock`
 * but uses the runtime `UserCodeInputField` (8 alpha slots, XXXX-XXXX) so
 * the admin sees exactly what end-users will. `runtime={false}` keeps it
 * from reading any `?user_code=` prefill while editing.
 */
function UserCodeInputBlock({ node, isSelected }: { node: BuilderNode; isSelected: boolean }) {
  const label = (node.props.label as string) ?? ''
  const helperText = (node.props.helperText as string) ?? ''

  return (
    <div
      data-fk-id={node.id}
      style={{
        ...chromeStyle(isSelected),
        display: 'flex',
        flexDirection: 'column',
        gap: 6,
        alignItems: 'center',
        pointerEvents: 'none',
        ...orderStyle(node),
      }}
    >
      {label ? <label style={inputLabelStyle()}>{label}</label> : null}
      <UserCodeInputField disabled name={resolveInputName(node) ?? 'user_code'} runtime={false} />
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

/**
 * Forgot password link with inline content editing, mirrors the editable
 * heading/text blocks. Click is suppressed via `e.preventDefault` so the
 * canvas doesn't navigate while the admin is composing — the SortableNode
 * wrapper's `onClick` (selection) still runs because we don't
 * `stopPropagation`.
 */
function EditableForgotPasswordLink({
  node,
  isSelected,
}: {
  node: BuilderNode
  isSelected: boolean
}) {
  const { updateNode } = useBuilderContext()
  const style = mergeStyles(forgotPasswordLinkStyle(node), isSelected)

  if (isSelected) {
    return (
      <a
        data-fk-id={node.id}
        href='#'
        style={style}
        onPointerDown={(e) => e.stopPropagation()}
        onClick={(e) => {
          e.preventDefault()
          e.stopPropagation()
        }}
      >
        <InlineTextEditor
          content={node.content ?? ''}
          onChange={(value) => updateNode(node.id, { content: value })}
        />
      </a>
    )
  }
  return (
    <a
      data-fk-id={node.id}
      href='#'
      style={style}
      onClick={(e) => e.preventDefault()}
    >
      {node.content ?? 'Forgot password?'}
    </a>
  )
}

/**
 * "Back to login" link — shares the inline-text-editor + click-suppression
 * pattern with `EditableForgotPasswordLink`. Same styling token (link
 * tokens from the theme), different default content + href.
 */
function EditableBackToLoginLink({
  node,
  isSelected,
}: {
  node: BuilderNode
  isSelected: boolean
}) {
  const { updateNode } = useBuilderContext()
  const style = mergeStyles(forgotPasswordLinkStyle(node), isSelected)

  if (isSelected) {
    return (
      <a
        data-fk-id={node.id}
        href='#'
        style={style}
        onPointerDown={(e) => e.stopPropagation()}
        onClick={(e) => {
          e.preventDefault()
          e.stopPropagation()
        }}
      >
        <InlineTextEditor
          content={node.content ?? ''}
          onChange={(value) => updateNode(node.id, { content: value })}
        />
      </a>
    )
  }
  return (
    <a
      data-fk-id={node.id}
      href='#'
      style={style}
      onClick={(e) => e.preventDefault()}
    >
      {node.content ?? 'Back to login'}
    </a>
  )
}

/**
 * "Don't have an account? Sign up" link. Same editor pattern as the other
 * auth-flow nav links — shared styling, different default content + href.
 */
function EditableRegisterLink({
  node,
  isSelected,
}: {
  node: BuilderNode
  isSelected: boolean
}) {
  const { updateNode } = useBuilderContext()
  const style = mergeStyles(forgotPasswordLinkStyle(node), isSelected)

  if (isSelected) {
    return (
      <a
        data-fk-id={node.id}
        href='#'
        style={style}
        onPointerDown={(e) => e.stopPropagation()}
        onClick={(e) => {
          e.preventDefault()
          e.stopPropagation()
        }}
      >
        <InlineTextEditor
          content={node.content ?? ''}
          onChange={(value) => updateNode(node.id, { content: value })}
        />
      </a>
    )
  }
  return (
    <a
      data-fk-id={node.id}
      href='#'
      style={style}
      onClick={(e) => e.preventDefault()}
    >
      {node.content ?? 'Don\u2019t have an account? Sign up'}
    </a>
  )
}

function PageContentSlot({ isSelected }: { node: BuilderNode; isSelected: boolean }) {
  return (
    <div
      style={{
        ...chromeStyle(isSelected),
        // Mirror the runtime stretch behaviour (`<div data-portal-page-content>`
        // is rendered with `width: 100%` + `align-self: stretch`), otherwise
        // the layout builder paints a content-width placeholder while the
        // live portal renders full-width — and the admin can't tell the two
        // apart until they ship.
        width: '100%',
        alignSelf: 'stretch',
        boxSizing: 'border-box',
        minHeight: 120,
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
