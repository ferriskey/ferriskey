import { useTranslation } from 'react-i18next'
import { defaultTheme, usePortalThemeContext } from '../../context/portal-theme-context'
import { ColorPicker } from '../controls/color-picker'
import { ValueSlider } from '../controls/value-slider'
import { PanelHeader, PanelSection } from './section'
import { UNIT_PX } from './units'

const D = defaultTheme

const TOKENS = {
  background: 'pageBackground',
  error: 'error',
  fieldGap: 'fieldGap',
  sectionGap: 'sectionGap',
} as const

export function PagePanel() {
  const { t } = useTranslation('portal')
  const { theme, setColor, setSpacing } = usePortalThemeContext()
  const { colors, spacing } = theme

  return (
    <div className='flex flex-col'>
      <PanelHeader
        title={t('builder.panel.page.title')}
        description={t('builder.panel.page.description')}
      />

      <PanelSection title={t('builder.panel.page.section.surface')}>
        <ColorPicker
          label={t('builder.control.background')}
          value={colors.pageBackground}
          defaultValue={D.colors.pageBackground}
          onChange={(v) => setColor(TOKENS.background, v)}
        />
      </PanelSection>

      <PanelSection title={t('builder.panel.page.section.feedback')}>
        <ColorPicker
          label={t('builder.control.error')}
          value={colors.error}
          defaultValue={D.colors.error}
          onChange={(v) => setColor(TOKENS.error, v)}
        />
      </PanelSection>

      <PanelSection title={t('builder.panel.page.section.spacing')}>
        <ValueSlider
          label={t('builder.control.field_gap')}
          value={spacing.fieldGap}
          defaultValue={D.spacing.fieldGap}
          onChange={(v) => setSpacing(TOKENS.fieldGap, v)}
          min={0}
          max={48}
          unit={UNIT_PX}
        />
        <ValueSlider
          label={t('builder.control.section_gap')}
          value={spacing.sectionGap}
          defaultValue={D.spacing.sectionGap}
          onChange={(v) => setSpacing(TOKENS.sectionGap, v)}
          min={0}
          max={64}
          unit={UNIT_PX}
        />
      </PanelSection>
    </div>
  )
}
