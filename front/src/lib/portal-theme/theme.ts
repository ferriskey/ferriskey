import type { Schemas } from '@/api/api.client'

export type ThemeColors = Required<NonNullable<Schemas.ThemeColors>>
export type ThemeFontStyle = Schemas.ThemeFontStyle
export type ThemeFontLinkStyle = Schemas.ThemeFontLinkStyle
export type ThemeFonts = Required<NonNullable<Schemas.ThemeFonts>>
export type ThemeBorders = Required<NonNullable<Schemas.ThemeBorders>>
export type ThemeSpacing = Required<NonNullable<Schemas.ThemeSpacing>>
export type ThemeShadow = Schemas.ThemeShadow

export type PortalThemeConfig = {
  colors: ThemeColors
  fonts: ThemeFonts
  borders: ThemeBorders
  spacing: ThemeSpacing
}

export const defaultTheme: PortalThemeConfig = {
  colors: {
    primaryButton: '#635dff',
    primaryButtonLabel: '#ffffff',
    secondaryButton: '#ffffff',
    secondaryButtonLabel: '#1e212a',
    socialButtonBackground: '#ffffff',
    socialButtonLabel: '#1e212a',
    socialButtonBorder: '#d1d5db',
    magicLinkButtonBackground: '#ffffff',
    magicLinkButtonLabel: '#1e212a',
    magicLinkButtonBorder: '#d1d5db',
    passkeyButtonBackground: '#ffffff',
    passkeyButtonLabel: '#1e212a',
    passkeyButtonBorder: '#d1d5db',
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
    socialButtonBorderWeight: 1,
    magicLinkButtonBorderWeight: 1,
    passkeyButtonBorderWeight: 1,
    inputRadius: 3,
    inputBorderWeight: 1,
    widgetRadius: 5,
    widgetBorderWeight: 1,
    // Mirrors the Rust default — themes that haven't been saved yet still
    // render with the pronounced shadow + 1px outline so the preview
    // matches what the backend will persist on first save.
    widgetShadow: 'large',
  },
  spacing: {
    widgetPadding: 24,
    fieldGap: 16,
    sectionGap: 24,
  },
}

export function mergeWithDefaults(partial: Schemas.PortalThemeConfig | undefined): PortalThemeConfig {
  return {
    colors: { ...defaultTheme.colors, ...(partial?.colors ?? {}) },
    fonts: { ...defaultTheme.fonts, ...(partial?.fonts ?? {}) },
    borders: { ...defaultTheme.borders, ...(partial?.borders ?? {}) },
    spacing: { ...defaultTheme.spacing, ...(partial?.spacing ?? {}) },
  }
}

// Tuned to match the FerrisKey default admin design — modern cards want
// a soft, well-dispersed elevation rather than a tight 1-2px drop. Each
// preset combines a wide ambient layer with a tighter contact layer so
// the card reads as detached from the page without feeling heavy.
const SHADOW_TO_CSS: Record<ThemeShadow, string> = {
  none: 'none',
  small: '0 1px 2px rgba(0,0,0,0.05), 0 4px 8px -2px rgba(0,0,0,0.05)',
  large: '0 4px 14px rgba(0,0,0,0.06), 0 12px 32px -4px rgba(0,0,0,0.08)',
}

export function themeToCssVars(theme: PortalThemeConfig): Record<string, string> {
  const { colors, fonts, borders, spacing } = theme

  return {
    '--fk-color-primary-button': colors.primaryButton,
    '--fk-color-primary-button-label': colors.primaryButtonLabel,
    '--fk-color-secondary-button': colors.secondaryButton,
    '--fk-color-secondary-button-label': colors.secondaryButtonLabel,
    '--fk-color-social-button-bg': colors.socialButtonBackground,
    '--fk-color-social-button-label': colors.socialButtonLabel,
    '--fk-color-social-button-border': colors.socialButtonBorder,
    '--fk-color-magic-link-button-bg': colors.magicLinkButtonBackground,
    '--fk-color-magic-link-button-label': colors.magicLinkButtonLabel,
    '--fk-color-magic-link-button-border': colors.magicLinkButtonBorder,
    '--fk-color-passkey-button-bg': colors.passkeyButtonBackground,
    '--fk-color-passkey-button-label': colors.passkeyButtonLabel,
    '--fk-color-passkey-button-border': colors.passkeyButtonBorder,
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
    '--fk-border-social-button': `${borders.socialButtonBorderWeight}px`,
    '--fk-border-magic-link-button': `${borders.magicLinkButtonBorderWeight}px`,
    '--fk-border-passkey-button': `${borders.passkeyButtonBorderWeight}px`,
    '--fk-radius-input': `${borders.inputRadius}px`,
    '--fk-border-input': `${borders.inputBorderWeight}px`,
    '--fk-radius-widget': `${borders.widgetRadius}px`,
    '--fk-border-widget': `${borders.widgetBorderWeight}px`,
    '--fk-shadow-widget': SHADOW_TO_CSS[borders.widgetShadow],

    '--fk-spacing-widget-padding': `${spacing.widgetPadding}px`,
    '--fk-spacing-field-gap': `${spacing.fieldGap}px`,
    '--fk-spacing-section-gap': `${spacing.sectionGap}px`,
  }
}
