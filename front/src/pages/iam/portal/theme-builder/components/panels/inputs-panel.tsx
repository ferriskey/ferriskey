import { useTranslation } from 'react-i18next'
import { defaultTheme, usePortalThemeContext } from '../../context/portal-theme-context'
import { ValueSlider } from '../controls/value-slider'
import { PanelHeader, PanelSection } from './section'
import { UNIT_PERCENT, UNIT_PX } from './units'

const D = defaultTheme

const TOKENS = {
  radius: 'inputRadius',
  borderWeight: 'inputBorderWeight',
  font: 'inputLabels',
} as const

export function InputsPanel() {
  const { t } = useTranslation('portal')
  const { theme, setBorder, setFont } = usePortalThemeContext()
  const { borders, fonts } = theme

  return (
    <div className='flex flex-col'>
      <PanelHeader
        title={t('builder.panel.inputs.title')}
        description={t('builder.panel.inputs.description')}
      />

      <PanelSection title={t('builder.panel.inputs.section.shape')}>
        <ValueSlider
          label={t('builder.control.radius')}
          value={borders.inputRadius}
          defaultValue={D.borders.inputRadius}
          onChange={(v) => setBorder(TOKENS.radius, v)}
          min={0}
          max={32}
          unit={UNIT_PX}
        />
        <ValueSlider
          label={t('builder.control.border')}
          value={borders.inputBorderWeight}
          defaultValue={D.borders.inputBorderWeight}
          onChange={(v) => setBorder(TOKENS.borderWeight, v)}
          min={0}
          max={6}
          unit={UNIT_PX}
        />
      </PanelSection>

      <PanelSection title={t('builder.panel.inputs.section.label_typography')} defaultOpen={false}>
        <ValueSlider
          label={t('builder.control.weight')}
          value={fonts.inputLabels.weight}
          defaultValue={D.fonts.inputLabels.weight}
          onChange={(weight) => setFont(TOKENS.font, { ...fonts.inputLabels, weight })}
          min={100}
          max={900}
          step={100}
        />
        <ValueSlider
          label={t('builder.control.size')}
          value={fonts.inputLabels.sizePct}
          defaultValue={D.fonts.inputLabels.sizePct}
          onChange={(sizePct) => setFont(TOKENS.font, { ...fonts.inputLabels, sizePct })}
          min={50}
          max={200}
          step={2.5}
          unit={UNIT_PERCENT}
        />
      </PanelSection>
    </div>
  )
}
