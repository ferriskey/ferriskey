import { defaultTheme, usePortalThemeContext } from '../../context/portal-theme-context'
import { ValueSlider } from '../controls/value-slider'
import { PanelHeader, PanelSection } from './section'

const D = defaultTheme

export function InputsPanel() {
  const { theme, setBorder, setFont } = usePortalThemeContext()
  const { borders, fonts } = theme

  return (
    <div className='flex flex-col'>
      <PanelHeader
        title='Inputs'
        description='Form fields (email, password, OTP). Background and text colors follow the page tokens; tune shape and label typography here.'
      />

      <PanelSection title='Shape'>
        <ValueSlider
          label='Radius'
          value={borders.inputRadius}
          defaultValue={D.borders.inputRadius}
          onChange={(v) => setBorder('inputRadius', v)}
          min={0}
          max={32}
          unit='px'
        />
        <ValueSlider
          label='Border'
          value={borders.inputBorderWeight}
          defaultValue={D.borders.inputBorderWeight}
          onChange={(v) => setBorder('inputBorderWeight', v)}
          min={0}
          max={6}
          unit='px'
        />
      </PanelSection>

      <PanelSection title='Label typography' defaultOpen={false}>
        <ValueSlider
          label='Weight'
          value={fonts.inputLabels.weight}
          defaultValue={D.fonts.inputLabels.weight}
          onChange={(weight) => setFont('inputLabels', { ...fonts.inputLabels, weight })}
          min={100}
          max={900}
          step={100}
        />
        <ValueSlider
          label='Size'
          value={fonts.inputLabels.sizePct}
          defaultValue={D.fonts.inputLabels.sizePct}
          onChange={(sizePct) => setFont('inputLabels', { ...fonts.inputLabels, sizePct })}
          min={50}
          max={200}
          step={2.5}
          unit='%'
        />
      </PanelSection>
    </div>
  )
}
