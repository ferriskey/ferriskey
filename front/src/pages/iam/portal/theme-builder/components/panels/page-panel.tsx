import { defaultTheme, usePortalThemeContext } from '../../context/portal-theme-context'
import { ColorPicker } from '../controls/color-picker'
import { ValueSlider } from '../controls/value-slider'
import { PanelHeader, PanelSection } from './section'

const D = defaultTheme

export function PagePanel() {
  const { theme, setColor, setSpacing } = usePortalThemeContext()
  const { colors, spacing } = theme

  return (
    <div className='flex flex-col'>
      <PanelHeader
        title='Page'
        description='Outermost surface behind the widget plus global feedback colors and spacing.'
      />

      <PanelSection title='Surface'>
        <ColorPicker
          label='Background'
          value={colors.pageBackground}
          defaultValue={D.colors.pageBackground}
          onChange={(v) => setColor('pageBackground', v)}
        />
      </PanelSection>

      <PanelSection title='Feedback'>
        <ColorPicker
          label='Error'
          value={colors.error}
          defaultValue={D.colors.error}
          onChange={(v) => setColor('error', v)}
        />
      </PanelSection>

      <PanelSection title='Spacing'>
        <ValueSlider
          label='Field gap'
          value={spacing.fieldGap}
          defaultValue={D.spacing.fieldGap}
          onChange={(v) => setSpacing('fieldGap', v)}
          min={0}
          max={48}
          unit='px'
        />
        <ValueSlider
          label='Section gap'
          value={spacing.sectionGap}
          defaultValue={D.spacing.sectionGap}
          onChange={(v) => setSpacing('sectionGap', v)}
          min={0}
          max={64}
          unit='px'
        />
      </PanelSection>
    </div>
  )
}
