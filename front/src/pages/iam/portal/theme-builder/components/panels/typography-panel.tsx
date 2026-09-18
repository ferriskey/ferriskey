import { useTranslation } from 'react-i18next'
import { Input } from '@/components/ui/input'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { defaultTheme, usePortalThemeContext } from '../../context/portal-theme-context'
import type {
  ThemeFontLinkStyle,
  ThemeFontStyle,
  ThemeFonts,
} from '@/lib/portal-theme/theme'
import { ColorPicker } from '../controls/color-picker'
import { ControlRow } from '../controls/control-row'
import { ValueSlider } from '../controls/value-slider'
import { PanelHeader, PanelSection } from './section'
import { UNIT_PERCENT, UNIT_PX } from './units'

const D = defaultTheme

const TOKENS = {
  url: 'url',
  baseSize: 'baseSize',
  bodyText: 'bodyText',
  links: 'links',
} as const

const FONT_URL_PLACEHOLDER = 'https://fonts.googleapis.com/…'

const LINK_STYLES: ThemeFontLinkStyle['style'][] = ['normal', 'underline']

type StyleKey = Extract<keyof ThemeFonts, 'title' | 'subtitle' | 'body'>

const STYLE_FIELDS: Array<{ key: StyleKey; labelKey: string }> = [
  { key: 'title', labelKey: 'builder.panel.typography.style.title' },
  { key: 'subtitle', labelKey: 'builder.panel.typography.style.subtitle' },
  { key: 'body', labelKey: 'builder.panel.typography.style.body' },
]

function FontStyleSection({
  label,
  value,
  defaultStyle,
  onChange,
  defaultOpen = false,
}: {
  label: string
  value: ThemeFontStyle
  defaultStyle: ThemeFontStyle
  onChange: (next: ThemeFontStyle) => void
  defaultOpen?: boolean
}) {
  const { t } = useTranslation('portal')

  return (
    <PanelSection title={label} defaultOpen={defaultOpen}>
      <ValueSlider
        label={t('builder.control.weight')}
        value={value.weight}
        defaultValue={defaultStyle.weight}
        onChange={(weight) => onChange({ ...value, weight })}
        min={100}
        max={900}
        step={100}
      />
      <ValueSlider
        label={t('builder.control.size')}
        value={value.sizePct}
        defaultValue={defaultStyle.sizePct}
        onChange={(sizePct) => onChange({ ...value, sizePct })}
        min={50}
        max={200}
        step={2.5}
        unit={UNIT_PERCENT}
      />
    </PanelSection>
  )
}

export function TypographyPanel() {
  const { t } = useTranslation('portal')
  const { theme, setColor, setFont } = usePortalThemeContext()
  const { colors, fonts } = theme

  const handleLinkChange = (next: ThemeFontLinkStyle) => setFont(TOKENS.links, next)

  return (
    <div className='flex flex-col'>
      <PanelHeader
        title={t('builder.panel.typography.title')}
        description={t('builder.panel.typography.description')}
      />

      <PanelSection title={t('builder.panel.typography.section.family')}>
        <ControlRow
          label={t('builder.control.custom_url')}
          modified={(fonts.url ?? null) !== (D.fonts.url ?? null)}
          onReset={() => setFont(TOKENS.url, D.fonts.url)}
        >
          <Input
            value={fonts.url ?? ''}
            placeholder={FONT_URL_PLACEHOLDER}
            onChange={(e) => setFont(TOKENS.url, e.target.value || null)}
            className='h-7 text-[11px]'
          />
        </ControlRow>
        <ValueSlider
          label={t('builder.control.base_size')}
          value={fonts.baseSize}
          defaultValue={D.fonts.baseSize}
          onChange={(baseSize) => setFont(TOKENS.baseSize, baseSize)}
          min={12}
          max={20}
          unit={UNIT_PX}
        />
      </PanelSection>

      <PanelSection title={t('builder.panel.typography.section.body_color')}>
        <ColorPicker
          label={t('builder.control.default_text')}
          value={colors.bodyText}
          defaultValue={D.colors.bodyText}
          onChange={(v) => setColor(TOKENS.bodyText, v)}
        />
      </PanelSection>

      {STYLE_FIELDS.map(({ key, labelKey }) => (
        <FontStyleSection
          key={key}
          label={t(labelKey)}
          value={fonts[key]}
          defaultStyle={D.fonts[key]}
          onChange={(next) => setFont(key, next)}
          defaultOpen={key === 'title'}
        />
      ))}

      <PanelSection title={t('builder.panel.typography.section.links')} defaultOpen={false}>
        <ColorPicker
          label={t('builder.control.color')}
          value={colors.links}
          defaultValue={D.colors.links}
          onChange={(v) => setColor(TOKENS.links, v)}
        />
        <ValueSlider
          label={t('builder.control.weight')}
          value={fonts.links.weight}
          defaultValue={D.fonts.links.weight}
          onChange={(weight) => handleLinkChange({ ...fonts.links, weight })}
          min={100}
          max={900}
          step={100}
        />
        <ValueSlider
          label={t('builder.control.size')}
          value={fonts.links.sizePct}
          defaultValue={D.fonts.links.sizePct}
          onChange={(sizePct) => handleLinkChange({ ...fonts.links, sizePct })}
          min={50}
          max={200}
          step={2.5}
          unit={UNIT_PERCENT}
        />
        <ControlRow
          label={t('builder.control.decoration')}
          modified={fonts.links.style !== D.fonts.links.style}
          onReset={() => handleLinkChange({ ...fonts.links, style: D.fonts.links.style })}
        >
          <Select
            value={fonts.links.style}
            onValueChange={(style: ThemeFontLinkStyle['style']) =>
              handleLinkChange({ ...fonts.links, style })
            }
          >
            <SelectTrigger className='h-7 w-full text-xs'>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {LINK_STYLES.map((style) => (
                <SelectItem key={style} value={style}>
                  {t(`builder.link_style.${style}`)}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </ControlRow>
      </PanelSection>
    </div>
  )
}
