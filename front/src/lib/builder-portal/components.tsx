import {
  AtSign,
  Box,
  CheckSquare,
  Globe,
  Heading as HeadingIcon,
  Image as ImageIcon,
  KeyRound,
  LayoutTemplate,
  LockKeyhole,
  Minus,
  MousePointerClick,
  MoveVertical,
  Square,
  TextCursorInput,
  Type,
} from 'lucide-react'
import type { BuilderNode, ComponentDefinition } from '../builder-core'

const ALL_CHILDREN = [
  'container',
  'div',
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
  'identity_providers',
  'page-content',
]

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
]

/** Block types that are specialized for a portal page (not generic layout). */
export const REQUIRED_BLOCK_TYPES = new Set([
  'email_input',
  'password_input',
  'totp_input',
  'submit_button',
  'identity_providers',
])

/** Block types that only make sense in a layout tree, never in a page tree. */
export const LAYOUT_ONLY_BLOCK_TYPES = new Set(['page-content'])

const DEFAULT_CONTENT: Partial<Record<string, string>> = {
  heading: 'Welcome',
  text: 'Sign in to your account.',
  button: 'Continue',
  submit_button: 'Continue',
}

export function getDefaultNode(type: string): Omit<BuilderNode, 'id'> {
  const def = portalComponents.find((c) => c.type === type)
  return {
    type,
    props: { ...(def?.defaultProps ?? {}) },
    styles: { ...(def?.defaultStyles ?? {}) },
    children: [],
    content: def?.hasContent ? (DEFAULT_CONTENT[type] ?? '') : undefined,
  }
}
