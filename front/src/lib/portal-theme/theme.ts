import type { Schemas } from '@/api/api.client'
import type { BuilderNode } from '@/lib/builder-core'

export type ThemeColors = Required<NonNullable<Schemas.ThemeColors>>
export type ThemeFontStyle = Schemas.ThemeFontStyle
export type ThemeFontLinkStyle = Schemas.ThemeFontLinkStyle
export type ThemeFonts = Required<NonNullable<Schemas.ThemeFonts>>
export type ThemeBorders = Required<NonNullable<Schemas.ThemeBorders>>
export type ThemeSpacing = Required<NonNullable<Schemas.ThemeSpacing>>
export type ThemeShadow = Schemas.ThemeShadow

// The editor keeps a stored dark palette as it is.
// A theme without one has no such key.
type StoredDarkPalette = { darkColors: Schemas.ThemeColors } | Record<never, never>

export type PortalThemeConfig = {
  colors: ThemeColors
  fonts: ThemeFonts
  borders: ThemeBorders
  spacing: ThemeSpacing
} & StoredDarkPalette

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
  const storedDark = partial ? partial.darkColors : null
  return {
    colors: { ...defaultTheme.colors, ...(partial?.colors ?? {}) },
    fonts: { ...defaultTheme.fonts, ...(partial?.fonts ?? {}) },
    borders: { ...defaultTheme.borders, ...(partial?.borders ?? {}) },
    spacing: { ...defaultTheme.spacing, ...(partial?.spacing ?? {}) },
    ...(storedDark ? { darkColors: storedDark } : {}),
  }
}

/**
 * Swaps `colors` for `darkColors`, when the theme has them.
 * The result can miss dark tokens.
 * Pass it to `mergeWithDefaults`, which gives them defaults.
 *
 * @example
 * withDarkPalette({ darkColors: { bodyText: '#fff' } }) // { colors: { bodyText: '#fff' }, … }
 */
export function withDarkPalette(tokens: Schemas.PortalThemeConfig): Schemas.PortalThemeConfig {
  if (!tokens.darkColors) return tokens
  return { ...tokens, colors: tokens.darkColors }
}

const COLOR_VAR_PREFIX = '--fk-color-'
const LIGHT_VAR_PREFIX = '--fk-light-color-'
// The blocks whose style reads a `backgroundColor` prop.
const SURFACE_BLOCKS = new Set(['container', 'flex', 'grid', 'div', 'card'])

const CLEAR_KEYWORDS = new Set(['transparent', 'initial', 'unset'])
// Hex colour with zero alpha: `#rgb0` or `#rrggbb00`.
const CLEAR_HEX = /^#(?:[0-9a-f]{3}0|[0-9a-f]{6}00)$/
// Matches a colour function and captures its arguments.
const COLOR_FUNCTION = /^(?:rgba?|hsla?|hwb|lab|lch|oklab|oklch|color)\((.*)\)$/
const LEGACY_PARTS_WITH_ALPHA = 4

function lightVarName(name: string): string {
  return name.replace(COLOR_VAR_PREFIX, LIGHT_VAR_PREFIX)
}

// The alpha of a colour function, `1` when it has none.
function alphaOf(color: string): string {
  const args = color.replace(COLOR_FUNCTION, '$1')
  if (args.includes('/')) return args.slice(args.lastIndexOf('/') + 1)
  const parts = args.split(',')
  if (parts.length !== LEGACY_PARTS_WITH_ALPHA) return '1'
  const [, , , alpha] = parts
  return alpha
}

// Blank text changes no background in the generated CSS.
function isApplicable(color: unknown): color is string {
  return typeof color === 'string' && color.trim() !== ''
}

// A fully transparent colour paints nothing.
function isSurfaceColor(color: string): boolean {
  const text = color.trim().toLowerCase()
  if (CLEAR_KEYWORDS.has(text) || CLEAR_HEX.test(text)) return false
  if (!COLOR_FUNCTION.test(text)) return true
  const alpha = Number.parseFloat(alphaOf(text))
  return Number.isNaN(alpha) || alpha > 0
}

type SurfacePalette = { apply: string; release: string }
type SurfaceStep = { width: number; surface: boolean }

// Word characters and `-`: safe in a quoted selector.
const PLAIN_ID = /^[\w-]+$/

// Widths where a visible background starts or stops, narrowest first.
function surfaceSteps(node: BuilderNode, widths: Record<string, number>): SurfaceStep[] {
  const layers = node.breakpoints ? Object.entries(node.breakpoints) : []
  return layers
    .flatMap(([name, props]) => {
      const color = props.backgroundColor
      if (!Number.isFinite(widths[name]) || !isApplicable(color)) return []
      return [{ width: widths[name], surface: isSurfaceColor(color) }]
    })
    .sort((a, b) => a.width - b.width)
}

function surfaceRules(
  node: BuilderNode,
  palette: SurfacePalette,
  widths: Record<string, number>,
): string[] {
  if (!SURFACE_BLOCKS.has(node.type) || !PLAIN_ID.test(node.id)) return []
  const rule = (declarations: string): string => `[data-fk-id="${node.id}"] { ${declarations} }`
  const base = node.props.backgroundColor
  const baseSurface = isApplicable(base) && isSurfaceColor(base)
  // A card's breakpoint overrides never reach the generated CSS.
  const steps = node.type === 'card' ? [] : surfaceSteps(node, widths)
  // `surfacesBefore[i]` is the state before step `i`.
  const surfacesBefore = [baseSurface, ...steps.map((step) => step.surface)]
  const changes = steps.filter((step, index) => step.surface !== surfacesBefore[index])
  return [
    ...(baseSurface ? [rule(palette.apply)] : []),
    ...changes.map((step) => {
      const declarations = step.surface ? palette.apply : palette.release
      return `@media (min-width: ${step.width}px) { ${rule(declarations)} }`
    }),
  ]
}

/**
 * Rules keeping the light palette inside blocks with a surface.
 * They switch at the widths where the background changes.
 * Empty when `palette.apply` is.
 *
 * @example
 * ownSurfaceCss(tree, { apply: 'a: b;', release: 'a: inherit;' }, { md: 768 })
 * // '[data-fk-id="box"] { a: b; }'
 */
export function ownSurfaceCss(
  tree: BuilderNode[],
  palette: SurfacePalette,
  widths: Record<string, number>,
): string {
  if (!palette.apply) return ''
  const rulesOf = (nodes: BuilderNode[]): string[] =>
    nodes.flatMap((node) => [...surfaceRules(node, palette, widths), ...rulesOf(node.children)])
  return rulesOf(tree).join('\n')
}

/**
 * The light palette, for blocks with their own surface.
 * `vars` hold its colours under `--fk-light-color-*` names.
 * `apply` maps them back onto a block.
 * `release` takes the colours of its parent again.
 *
 * @example
 * ownSurfaceTheme(lightTheme).vars // { '--fk-light-color-body-text': '#1e212a', … }
 */
export function ownSurfaceTheme(light: PortalThemeConfig): SurfacePalette & {
  vars: Record<string, string>
} {
  const colors = Object.entries(themeToCssVars(light)).filter(([name]) =>
    name.startsWith(COLOR_VAR_PREFIX),
  )
  return {
    vars: Object.fromEntries(colors.map(([name, value]) => [lightVarName(name), value])),
    apply: colors.map(([name]) => `${name}: var(${lightVarName(name)});`).join(' '),
    release: colors.map(([name]) => `${name}: inherit;`).join(' '),
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

const OUTLINE_PERCENT = { floatingInput: 18, field: 15, card: 12 }

/**
 * Outline colour for a role, drawn in the outline ink.
 * Without the variable, it is the black outline of old.
 *
 * @example
 * outlineColor('card') // 'color-mix(in srgb, var(--fk-color-outline-ink, #000000) 12%, transparent)'
 */
export function outlineColor(role: keyof typeof OUTLINE_PERCENT): string {
  return `color-mix(in srgb, var(--fk-color-outline-ink, #000000) ${OUTLINE_PERCENT[role]}%, transparent)`
}

/**
 * What a dark palette changes on top of `themeToCssVars`.
 * Outlines follow its body text, to show on dark widgets.
 * Spread it only when `theme` holds the `darkColors`.
 *
 * @example
 * darkPaletteCssVars(theme) // { '--fk-color-outline-ink': '#f5fafc' }
 */
export function darkPaletteCssVars(theme: PortalThemeConfig): Record<string, string> {
  return { '--fk-color-outline-ink': theme.colors.bodyText }
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
    '--fk-color-outline-ink': '#000000',

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
