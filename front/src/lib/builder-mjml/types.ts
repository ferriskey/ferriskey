/** Props common to most MJML components */
export interface MjmlBaseProps {
  'background-color'?: string
  'padding'?: string
  'padding-top'?: string
  'padding-bottom'?: string
  'padding-left'?: string
  'padding-right'?: string
  'css-class'?: string
}

export interface MjmlSectionProps extends MjmlBaseProps {
  'border'?: string
  'border-radius'?: string
  'direction'?: 'ltr' | 'rtl'
  'full-width'?: 'full-width' | ''
  'text-align'?: 'left' | 'center' | 'right'
}

export interface MjmlColumnProps extends MjmlBaseProps {
  'width'?: string
  'vertical-align'?: 'top' | 'middle' | 'bottom'
  'border'?: string
  'border-radius'?: string
}

export interface MjmlTextProps extends MjmlBaseProps {
  'color'?: string
  'font-family'?: string
  'font-size'?: string
  'font-weight'?: string
  'line-height'?: string
  'letter-spacing'?: string
  'align'?: 'left' | 'center' | 'right' | 'justify'
  'text-decoration'?: string
  'text-transform'?: 'uppercase' | 'lowercase' | 'capitalize' | 'none'
}

export interface MjmlImageProps extends MjmlBaseProps {
  'src'?: string
  'alt'?: string
  'href'?: string
  'width'?: string
  'height'?: string
  'align'?: 'left' | 'center' | 'right'
  'border-radius'?: string
}

export interface MjmlButtonProps extends MjmlBaseProps {
  'href'?: string
  'background-color'?: string
  'color'?: string
  'font-family'?: string
  'font-size'?: string
  'font-weight'?: string
  'border-radius'?: string
  'inner-padding'?: string
  'align'?: 'left' | 'center' | 'right'
  'width'?: string
}

export interface MjmlDividerProps extends MjmlBaseProps {
  'border-color'?: string
  'border-style'?: 'solid' | 'dashed' | 'dotted'
  'border-width'?: string
  'width'?: string
}

export interface MjmlSpacerProps {
  'height'?: string
  'css-class'?: string
}

export interface MjmlSocialProps extends MjmlBaseProps {
  'mode'?: 'horizontal' | 'vertical'
  'align'?: 'left' | 'center' | 'right'
  'icon-size'?: string
  'font-size'?: string
  'color'?: string
}

export interface MjmlHeroProps extends MjmlBaseProps {
  'background-url'?: string
  'background-width'?: string
  'background-height'?: string
  'background-position'?: string
  'mode'?: 'fixed-height' | 'fluid-height'
  'width'?: string
  'vertical-align'?: 'top' | 'middle' | 'bottom'
}

export interface MjmlWrapperProps extends MjmlBaseProps {
  'border'?: string
  'border-radius'?: string
  'full-width'?: 'full-width' | ''
  'text-align'?: 'left' | 'center' | 'right'
}

export interface MjmlTableProps extends MjmlBaseProps {
  'color'?: string
  'cellpadding'?: string
  'cellspacing'?: string
  'font-family'?: string
  'font-size'?: string
  'line-height'?: string
  'width'?: string
  'align'?: 'left' | 'center' | 'right'
}

export interface MjmlNavbarProps extends MjmlBaseProps {
  'align'?: 'left' | 'center' | 'right'
  'hamburger'?: 'hamburger'
  'ico-color'?: string
}

export type PreviewMode = 'desktop' | 'tablet' | 'mobile'

export const PREVIEW_WIDTHS: Record<PreviewMode, number> = {
  desktop: 600,
  tablet: 480,
  mobile: 320,
}
