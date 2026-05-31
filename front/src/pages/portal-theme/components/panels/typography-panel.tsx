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
} from '../../lib/theme'
import { ColorPicker } from '../controls/color-picker'
import { ControlRow } from '../controls/control-row'
import { ValueSlider } from '../controls/value-slider'
import { PanelHeader, PanelSection } from './section'

const D = defaultTheme

type StyleKey = Extract<keyof ThemeFonts, 'title' | 'subtitle' | 'body'>

const STYLE_FIELDS: Array<{ key: StyleKey; label: string }> = [
  { key: 'title', label: 'Title' },
  { key: 'subtitle', label: 'Subtitle' },
  { key: 'body', label: 'Body' },
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
  return (
    <PanelSection title={label} defaultOpen={defaultOpen}>
      <ValueSlider
        label='Weight'
        value={value.weight}
        defaultValue={defaultStyle.weight}
        onChange={(weight) => onChange({ ...value, weight })}
        min={100}
        max={900}
        step={100}
      />
      <ValueSlider
        label='Size'
        value={value.sizePct}
        defaultValue={defaultStyle.sizePct}
        onChange={(sizePct) => onChange({ ...value, sizePct })}
        min={50}
        max={200}
        step={2.5}
        unit='%'
      />
    </PanelSection>
  )
}

export function TypographyPanel() {
  const { theme, setColor, setFont } = usePortalThemeContext()
  const { colors, fonts } = theme

  const handleLinkChange = (next: ThemeFontLinkStyle) => setFont('links', next)

  return (
    <div className='flex flex-col'>
      <PanelHeader
        title='Typography'
        description='Font family, base size, and the heading/body/link styles used everywhere.'
      />

      <PanelSection title='Family'>
        <ControlRow
          label='Custom URL'
          modified={(fonts.url ?? null) !== (D.fonts.url ?? null)}
          onReset={() => setFont('url', D.fonts.url)}
        >
          <Input
            value={fonts.url ?? ''}
            placeholder='https://fonts.googleapis.com/…'
            onChange={(e) => setFont('url', e.target.value || null)}
            className='h-7 text-[11px]'
          />
        </ControlRow>
        <ValueSlider
          label='Base size'
          value={fonts.baseSize}
          defaultValue={D.fonts.baseSize}
          onChange={(baseSize) => setFont('baseSize', baseSize)}
          min={12}
          max={20}
          unit='px'
        />
      </PanelSection>

      <PanelSection title='Body color'>
        <ColorPicker
          label='Default text'
          value={colors.bodyText}
          defaultValue={D.colors.bodyText}
          onChange={(v) => setColor('bodyText', v)}
        />
      </PanelSection>

      {STYLE_FIELDS.map(({ key, label }) => (
        <FontStyleSection
          key={key}
          label={label}
          value={fonts[key]}
          defaultStyle={D.fonts[key]}
          onChange={(next) => setFont(key, next)}
          defaultOpen={key === 'title'}
        />
      ))}

      <PanelSection title='Links' defaultOpen={false}>
        <ColorPicker
          label='Color'
          value={colors.links}
          defaultValue={D.colors.links}
          onChange={(v) => setColor('links', v)}
        />
        <ValueSlider
          label='Weight'
          value={fonts.links.weight}
          defaultValue={D.fonts.links.weight}
          onChange={(weight) => handleLinkChange({ ...fonts.links, weight })}
          min={100}
          max={900}
          step={100}
        />
        <ValueSlider
          label='Size'
          value={fonts.links.sizePct}
          defaultValue={D.fonts.links.sizePct}
          onChange={(sizePct) => handleLinkChange({ ...fonts.links, sizePct })}
          min={50}
          max={200}
          step={2.5}
          unit='%'
        />
        <ControlRow
          label='Decoration'
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
              <SelectItem value='normal'>Normal</SelectItem>
              <SelectItem value='underline'>Underline</SelectItem>
            </SelectContent>
          </Select>
        </ControlRow>
      </PanelSection>
    </div>
  )
}
