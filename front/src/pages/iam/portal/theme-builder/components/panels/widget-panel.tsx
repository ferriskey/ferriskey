import { useTranslation } from 'react-i18next'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { defaultTheme, usePortalThemeContext } from '../../context/portal-theme-context'
import type { ThemeShadow } from '@/lib/portal-theme/theme'
import { ControlRow } from '../controls/control-row'
import { ColorPicker } from '../controls/color-picker'
import { ValueSlider } from '../controls/value-slider'
import { PanelHeader, PanelSection } from './section'
import { UNIT_PX } from './units'

const D = defaultTheme

const TOKENS = {
  background: 'widgetBackground',
  radius: 'widgetRadius',
  borderWeight: 'widgetBorderWeight',
  shadow: 'widgetShadow',
  padding: 'widgetPadding',
} as const

const SHADOWS: ThemeShadow[] = ['none', 'small', 'large']

export function WidgetPanel() {
  const { t } = useTranslation('portal')
  const { theme, setColor, setBorder, setSpacing } = usePortalThemeContext()
  const { colors, borders, spacing } = theme

  return (
    <div className='flex flex-col'>
      <PanelHeader
        title={t('builder.panel.widget.title')}
        description={t('builder.panel.widget.description')}
      />

      <PanelSection title={t('builder.panel.widget.section.surface')}>
        <ColorPicker
          label={t('builder.control.background')}
          value={colors.widgetBackground}
          defaultValue={D.colors.widgetBackground}
          onChange={(v) => setColor(TOKENS.background, v)}
        />
      </PanelSection>

      <PanelSection title={t('builder.panel.widget.section.shape')}>
        <ValueSlider
          label={t('builder.control.radius')}
          value={borders.widgetRadius}
          defaultValue={D.borders.widgetRadius}
          onChange={(v) => setBorder(TOKENS.radius, v)}
          min={0}
          max={32}
          unit={UNIT_PX}
        />
        <ValueSlider
          label={t('builder.control.border')}
          value={borders.widgetBorderWeight}
          defaultValue={D.borders.widgetBorderWeight}
          onChange={(v) => setBorder(TOKENS.borderWeight, v)}
          min={0}
          max={6}
          unit={UNIT_PX}
        />
        <ControlRow
          label={t('builder.control.shadow')}
          modified={borders.widgetShadow !== D.borders.widgetShadow}
          onReset={() => setBorder(TOKENS.shadow, D.borders.widgetShadow)}
        >
          <Select
            value={borders.widgetShadow}
            onValueChange={(value: ThemeShadow) => setBorder(TOKENS.shadow, value)}
          >
            <SelectTrigger className='h-7 w-full text-xs'>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {SHADOWS.map((shadow) => (
                <SelectItem key={shadow} value={shadow}>
                  {t(`builder.shadow.${shadow}`)}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </ControlRow>
      </PanelSection>

      <PanelSection title={t('builder.panel.widget.section.spacing')}>
        <ValueSlider
          label={t('builder.control.padding')}
          value={spacing.widgetPadding}
          defaultValue={D.spacing.widgetPadding}
          onChange={(v) => setSpacing(TOKENS.padding, v)}
          min={0}
          max={64}
          unit={UNIT_PX}
        />
      </PanelSection>
    </div>
  )
}
