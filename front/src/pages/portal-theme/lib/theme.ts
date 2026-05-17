import type { Schemas } from '@/api/api.client'

export type ThemeColors = Required<NonNullable<Schemas.ThemeColors>>
export type ThemeFontStyle = Schemas.ThemeFontStyle
export type ThemeFontLinkStyle = Schemas.ThemeFontLinkStyle
export type ThemeFonts = Required<NonNullable<Schemas.ThemeFonts>>
export type ThemeBorders = Required<NonNullable<Schemas.ThemeBorders>>
export type ThemeShadow = Schemas.ThemeShadow

export type PortalThemeConfig = {
  colors: ThemeColors
  fonts: ThemeFonts
  borders: ThemeBorders
}

export const defaultTheme: PortalThemeConfig = {
  colors: {
    primaryButton: '#635dff',
    primaryButtonLabel: '#ffffff',
    secondaryButton: '#ffffff',
    secondaryButtonLabel: '#1e212a',
    widgetBackground: '#ffffff',
    pageBackground: '#000000',
    bodyText: '#1e212a',
    links: '#635dff',
    error: '#d03c38',
  },
  fonts: {
    url: null,
    baseSize: 16,
    title: { weight: 600, sizePct: 150 },
    subtitle: { weight: 400, sizePct: 87.5 },
    body: { weight: 400, sizePct: 87.5 },
    buttons: { weight: 600, sizePct: 100 },
    inputLabels: { weight: 500, sizePct: 100 },
    links: { weight: 600, sizePct: 87.5, style: 'normal' },
  },
  borders: {
    buttonRadius: 3,
    buttonBorderWeight: 1,
    inputRadius: 3,
    inputBorderWeight: 1,
    widgetRadius: 5,
    widgetBorderWeight: 0,
    widgetShadow: 'small',
  },
}

export function mergeWithDefaults(partial: Schemas.PortalThemeConfig | undefined): PortalThemeConfig {
  return {
    colors: { ...defaultTheme.colors, ...(partial?.colors ?? {}) },
    fonts: { ...defaultTheme.fonts, ...(partial?.fonts ?? {}) },
    borders: { ...defaultTheme.borders, ...(partial?.borders ?? {}) },
  }
}

const SHADOW_TO_CSS: Record<ThemeShadow, string> = {
  none: 'none',
  small: '0 1px 2px rgba(0,0,0,0.06), 0 1px 3px rgba(0,0,0,0.1)',
  large: '0 10px 15px -3px rgba(0,0,0,0.1), 0 4px 6px -4px rgba(0,0,0,0.1)',
}

export function themeToCssVars(theme: PortalThemeConfig): Record<string, string> {
  const { colors, fonts, borders } = theme

  return {
    '--fk-color-primary-button': colors.primaryButton,
    '--fk-color-primary-button-label': colors.primaryButtonLabel,
    '--fk-color-secondary-button': colors.secondaryButton,
    '--fk-color-secondary-button-label': colors.secondaryButtonLabel,
    '--fk-color-widget-bg': colors.widgetBackground,
    '--fk-color-page-bg': colors.pageBackground,
    '--fk-color-body-text': colors.bodyText,
    '--fk-color-links': colors.links,
    '--fk-color-error': colors.error,

    '--fk-font-base-size': `${fonts.baseSize}px`,
    '--fk-font-title-size': `${(fonts.title.sizePct / 100) * fonts.baseSize}px`,
    '--fk-font-title-weight': String(fonts.title.weight),
    '--fk-font-subtitle-size': `${(fonts.subtitle.sizePct / 100) * fonts.baseSize}px`,
    '--fk-font-subtitle-weight': String(fonts.subtitle.weight),
    '--fk-font-body-size': `${(fonts.body.sizePct / 100) * fonts.baseSize}px`,
    '--fk-font-body-weight': String(fonts.body.weight),
    '--fk-font-button-size': `${(fonts.buttons.sizePct / 100) * fonts.baseSize}px`,
    '--fk-font-button-weight': String(fonts.buttons.weight),
    '--fk-font-input-label-size': `${(fonts.inputLabels.sizePct / 100) * fonts.baseSize}px`,
    '--fk-font-input-label-weight': String(fonts.inputLabels.weight),
    '--fk-font-link-size': `${(fonts.links.sizePct / 100) * fonts.baseSize}px`,
    '--fk-font-link-weight': String(fonts.links.weight),
    '--fk-font-link-decoration': fonts.links.style === 'underline' ? 'underline' : 'none',

    '--fk-radius-button': `${borders.buttonRadius}px`,
    '--fk-border-button': `${borders.buttonBorderWeight}px`,
    '--fk-radius-input': `${borders.inputRadius}px`,
    '--fk-border-input': `${borders.inputBorderWeight}px`,
    '--fk-radius-widget': `${borders.widgetRadius}px`,
    '--fk-border-widget': `${borders.widgetBorderWeight}px`,
    '--fk-shadow-widget': SHADOW_TO_CSS[borders.widgetShadow],
  }
}
