import {
  AtSign,
  Box,
  CheckSquare,
  CreditCard,
  Fingerprint,
  Globe,
  Heading as HeadingIcon,
  Image as ImageIcon,
  KeyRound,
  LayoutTemplate,
  LockKeyhole,
  Mail,
  Minus,
  MousePointerClick,
  MoveVertical,
  PanelBottom,
  PanelTop,
  Square,
  TextCursorInput,
  Type,
} from 'lucide-react'
import { generateNodeId, type BuilderNode, type ComponentDefinition } from '../builder-core'

const ALL_CHILDREN = [
  'container',
  'div',
  'card',
  'heading',
  'text',
  'image',
  'spacer',
  'divider',
  'button',
  'input',
  'email_input',
  'password_input',
  'totp_input',
  'submit_button',
  'magic_link_button',
  'passkey_button',
  'identity_providers',
  'page-content',
]

/**
 * Children allowed inside a Card slot (header / content / footer). Same as
 * `ALL_CHILDREN` minus the slot blocks themselves and `card` (nesting cards
 * inside a slot would defeat the centred layout the parent card provides).
 */
const CARD_SLOT_CHILDREN = ALL_CHILDREN.filter((t) => t !== 'card')

export const portalComponents: ComponentDefinition[] = [
  {
    type: 'container',
    label: 'Container',
    icon: <Box size={14} />,
    isContainer: true,
    allowedChildren: ALL_CHILDREN,
    defaultProps: {
      direction: 'column',
      align: 'stretch',
      gap: '12px',
      padding: '16px',
    },
    defaultStyles: {},
  },
  {
    type: 'div',
    label: 'Div',
    icon: <Square size={14} />,
    isContainer: true,
    allowedChildren: ALL_CHILDREN,
    defaultProps: {
      // Display switches between block / flex / grid layouts.
      display: 'block',
      // Position / positioning offsets.
      position: 'static',
      top: '',
      right: '',
      bottom: '',
      left: '',
      zIndex: '',
      // Size.
      width: '',
      height: '',
      minWidth: '',
      maxWidth: '',
      minHeight: '',
      maxHeight: '',
      // Spacing — no default padding so a fresh Div is a truly empty box.
      padding: '',
      margin: '',
      gap: '',
      // Visual.
      backgroundColor: '',
      borderRadius: '',
      overflow: 'visible',
      // Flex props (used when display === 'flex').
      direction: 'row',
      wrap: 'nowrap',
      justifyContent: 'flex-start',
      alignItems: 'stretch',
      alignContent: 'stretch',
      // Grid props (used when display === 'grid').
      templateColumns: 'repeat(2, 1fr)',
      templateRows: '',
      columnGap: '',
      rowGap: '',
      justifyItems: 'stretch',
      autoFlow: 'row',
    },
    defaultStyles: {},
  },
  // ShadCN-style Card: opinionated wrapper that auto-centres at a sensible
  // max width with the theme's widget tokens (background, radius, border,
  // shadow, padding). Drops three named slots so the author always gets the
  // header / content / footer split for free — the slots themselves are not
  // exposed in the library and only live as children of a card.
  {
    type: 'card',
    label: 'Card',
    icon: <CreditCard size={14} />,
    isContainer: true,
    allowedChildren: ['card-header', 'card-content', 'card-footer'],
    defaultProps: {
      maxWidth: '440px',
      // `auto` margins centre the card horizontally in its parent regardless
      // of whether the parent uses block, flex, or grid layout.
      align: 'center',
    },
    defaultStyles: {},
  },
  {
    type: 'card-header',
    label: 'Card header',
    icon: <PanelTop size={14} />,
    isContainer: true,
    allowedChildren: CARD_SLOT_CHILDREN,
    defaultProps: {
      textAlign: 'center',
      gap: '6px',
    },
    defaultStyles: {},
  },
  {
    type: 'card-content',
    label: 'Card content',
    icon: <LayoutTemplate size={14} />,
    isContainer: true,
    allowedChildren: CARD_SLOT_CHILDREN,
    defaultProps: {
      gap: '12px',
    },
    defaultStyles: {},
  },
  {
    type: 'card-footer',
    label: 'Card footer',
    icon: <PanelBottom size={14} />,
    isContainer: true,
    allowedChildren: CARD_SLOT_CHILDREN,
    defaultProps: {
      direction: 'row',
      justifyContent: 'flex-end',
      gap: '8px',
    },
    defaultStyles: {},
  },
  {
    type: 'heading',
    label: 'Heading',
    icon: <HeadingIcon size={14} />,
    hasContent: true,
    defaultProps: {
      level: '2',
      textAlign: 'center',
      fontWeight: '600',
    },
    defaultStyles: {},
  },
  {
    type: 'text',
    label: 'Text',
    icon: <Type size={14} />,
    hasContent: true,
    defaultProps: {
      textAlign: 'left',
      fontSize: 'var(--fk-font-base-size, 16px)',
      lineHeight: '1.5',
    },
    defaultStyles: {},
  },
  {
    type: 'image',
    label: 'Image',
    icon: <ImageIcon size={14} />,
    defaultProps: {
      src: '/logo_ferriskey.png',
      alt: 'Logo',
      width: '64px',
      align: 'center',
    },
    defaultStyles: {},
  },
  {
    type: 'spacer',
    label: 'Spacer',
    icon: <MoveVertical size={14} />,
    defaultProps: { height: '16px' },
    defaultStyles: {},
  },
  {
    type: 'divider',
    label: 'Divider',
    icon: <Minus size={14} />,
    defaultProps: {
      color: 'var(--fk-color-body-text, #d1d5db)',
      thickness: '1px',
      width: '100%',
    },
    defaultStyles: {},
  },
  {
    type: 'button',
    label: 'Button',
    icon: <MousePointerClick size={14} />,
    hasContent: true,
    defaultProps: {
      variant: 'primary',
      href: '#',
      fullWidth: 'true',
    },
    defaultStyles: {},
  },
  {
    type: 'input',
    label: 'Input',
    icon: <TextCursorInput size={14} />,
    defaultProps: {
      label: 'Username',
      placeholder: '',
      type: 'text',
    },
    defaultStyles: {},
  },
  {
    type: 'page-content',
    label: 'Page content',
    icon: <LayoutTemplate size={14} />,
    defaultProps: {},
    defaultStyles: {},
  },
  {
    type: 'email_input',
    label: 'Email input',
    icon: <AtSign size={14} />,
    defaultProps: {
      label: 'Email',
      placeholder: 'you@example.com',
      type: 'text',
      name: 'email',
    },
    defaultStyles: {},
  },
  {
    type: 'password_input',
    label: 'Password input',
    icon: <LockKeyhole size={14} />,
    defaultProps: {
      label: 'Password',
      placeholder: '',
      type: 'password',
      name: 'password',
    },
    defaultStyles: {},
  },
  {
    type: 'totp_input',
    label: 'TOTP input',
    icon: <KeyRound size={14} />,
    defaultProps: {
      label: 'One-time code',
      placeholder: '123 456',
      type: 'text',
      name: 'totp',
    },
    defaultStyles: {},
  },
  {
    type: 'submit_button',
    label: 'Submit button',
    icon: <CheckSquare size={14} />,
    hasContent: true,
    defaultProps: {
      variant: 'primary',
      fullWidth: 'true',
      submit: 'true',
    },
    defaultStyles: {},
  },
  {
    type: 'identity_providers',
    label: 'Identity providers',
    icon: <Globe size={14} />,
    defaultProps: {
      // Separator label displayed above the list ("Or continue with").
      separatorLabel: 'Or continue with',
      // Localizable button label prefix; the provider's display name is appended.
      buttonLabel: 'Continue with',
    },
    defaultStyles: {},
  },
  {
    type: 'magic_link_button',
    label: 'Magic link',
    icon: <Mail size={14} />,
    hasContent: true,
    defaultProps: {
      // Outline by default — magic link is an alternative auth path, not the
      // primary CTA (which is the submit button).
      variant: 'outline',
      fullWidth: 'true',
    },
    defaultStyles: {},
  },
  {
    type: 'passkey_button',
    label: 'Passkey',
    icon: <Fingerprint size={14} />,
    hasContent: true,
    defaultProps: {
      variant: 'outline',
      fullWidth: 'true',
    },
    defaultStyles: {},
  },
]

/** Block types that are specialized for a portal page (not generic layout). */
export const REQUIRED_BLOCK_TYPES = new Set([
  'email_input',
  'password_input',
  'totp_input',
  'submit_button',
  'identity_providers',
])

/**
 * Block types hidden from the component library. Either layout-tree only
 * (`page-content`), or named slots that exist solely as children of a parent
 * block (`card-header` / `card-content` / `card-footer` — pre-populated by
 * the Card's `getDefaultNode` and never authored standalone).
 */
export const LAYOUT_ONLY_BLOCK_TYPES = new Set([
  'page-content',
  'card-header',
  'card-content',
  'card-footer',
])

const DEFAULT_CONTENT: Partial<Record<string, string>> = {
  heading: 'Welcome',
  text: 'Sign in to your account.',
  button: 'Continue',
  submit_button: 'Continue',
  magic_link_button: 'Sign in with a magic link',
  passkey_button: 'Sign in with a passkey',
}

/**
 * Build a fully-formed `BuilderNode` (including `id`) of the given type using
 * the same defaults as `getDefaultNode`. Used by `getDefaultNode('card')` to
 * pre-populate the header/content/footer slots so the author drops a Card and
 * gets a working layout immediately — no need to add the three slots by hand.
 */
function buildNode(type: string): BuilderNode {
  const seed = getDefaultNode(type)
  return { ...seed, id: generateNodeId() }
}

export function getDefaultNode(type: string): Omit<BuilderNode, 'id'> {
  const def = portalComponents.find((c) => c.type === type)
  const base: Omit<BuilderNode, 'id'> = {
    type,
    props: { ...(def?.defaultProps ?? {}) },
    styles: { ...(def?.defaultStyles ?? {}) },
    children: [],
    content: def?.hasContent ? (DEFAULT_CONTENT[type] ?? '') : undefined,
  }

  // A Card is unusable without its three slots — pre-fill them so dropping a
  // Card produces the same shape ShadCN users expect (header / content /
  // footer all present, ready for blocks). The slot defaults are produced by
  // recursing through `buildNode`, which gives each one a stable id.
  if (type === 'card') {
    return { ...base, children: [buildNode('card-header'), buildNode('card-content'), buildNode('card-footer')] }
  }

  return base
}
