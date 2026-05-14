export interface PortalContainerProps {
  backgroundColor?: string
  padding?: string
  borderRadius?: string
  width?: string
  direction?: 'row' | 'column'
  align?: 'flex-start' | 'center' | 'flex-end' | 'stretch'
  gap?: string
}

export interface PortalHeadingProps {
  level?: '1' | '2' | '3' | '4'
  color?: string
  textAlign?: 'left' | 'center' | 'right'
  fontSize?: string
  fontWeight?: string
}

export interface PortalTextProps {
  color?: string
  textAlign?: 'left' | 'center' | 'right'
  fontSize?: string
  fontWeight?: string
  lineHeight?: string
}

export interface PortalImageProps {
  src?: string
  alt?: string
  width?: string
  height?: string
  align?: 'left' | 'center' | 'right'
  borderRadius?: string
}

export interface PortalSpacerProps {
  height?: string
}

export interface PortalDividerProps {
  color?: string
  thickness?: string
  width?: string
}

export interface PortalButtonProps {
  href?: string
  variant?: 'primary' | 'secondary' | 'outline'
  fullWidth?: 'true' | 'false'
}

export interface PortalInputProps {
  label?: string
  placeholder?: string
  type?: 'text' | 'email' | 'password'
  helperText?: string
}

export type PortalNodeType =
  | 'container'
  | 'heading'
  | 'text'
  | 'image'
  | 'spacer'
  | 'divider'
  | 'button'
  | 'input'
  | 'page-content'
