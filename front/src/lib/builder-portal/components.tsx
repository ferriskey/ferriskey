import {
  Box,
  Heading as HeadingIcon,
  Image as ImageIcon,
  LayoutTemplate,
  Minus,
  MousePointerClick,
  MoveVertical,
  TextCursorInput,
  Type,
} from 'lucide-react'
import type { BuilderNode, ComponentDefinition } from '../builder-core'

export const portalComponents: ComponentDefinition[] = [
  {
    type: 'container',
    label: 'Container',
    icon: <Box size={14} />,
    isContainer: true,
    allowedChildren: [
      'container',
      'heading',
      'text',
      'image',
      'spacer',
      'divider',
      'button',
      'input',
      'page-content',
    ],
    defaultProps: {
      direction: 'column',
      align: 'stretch',
      gap: '12px',
      padding: '16px',
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
]

const DEFAULT_CONTENT: Partial<Record<string, string>> = {
  heading: 'Welcome',
  text: 'Sign in to your account.',
  button: 'Continue',
}

export function getDefaultNode(type: string): Omit<BuilderNode, 'id'> {
  const def = portalComponents.find((c) => c.type === type)
  return {
    type,
    props: { ...(def?.defaultProps ?? {}) },
    styles: { ...(def?.defaultStyles ?? {}) },
    children: [],
    content: def?.hasContent ? DEFAULT_CONTENT[type] ?? '' : undefined,
  }
}
