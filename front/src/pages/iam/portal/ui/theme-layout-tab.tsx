import { useTranslation } from 'react-i18next'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { FieldRow, Section } from '@/components/kit'
import type { Schemas } from '@/api/api.client'

export const NO_LAYOUT = '__none__'

export interface ThemeLayoutTabProps {
  layouts: Schemas.PortalLayout[]
  layoutId: string
  savedLayoutId: string
  onLayoutChange: (value: string) => void
}

export default function ThemeLayoutTab({
  layouts,
  layoutId,
  savedLayoutId,
  onLayoutChange,
}: ThemeLayoutTabProps) {
  const { t } = useTranslation('portal')
  const selected = layouts.find((l) => l.id === layoutId)
  const detaching = layouts.find((l) => l.id === savedLayoutId && savedLayoutId !== layoutId)

  return (
    <Section
      title={t('detail.layout.title')}
      description={t('detail.layout.description')}
    >
      <FieldRow
        label={t('detail.layout.field.label')}
        description={t('detail.layout.field.description')}
      >
        <div className='max-w-sm'>
          <Select value={layoutId} onValueChange={onLayoutChange}>
            <SelectTrigger className='w-full'>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value={NO_LAYOUT}>{t('detail.layout.option_none')}</SelectItem>
              {layouts.map((layout) => (
                <SelectItem key={layout.id} value={layout.id}>
                  {layout.is_default
                    ? t('detail.layout.option_default', { name: layout.name })
                    : layout.name}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>

          {selected && (
            <p className='mt-1.5 text-xs text-neutral-400 dark:text-neutral-500'>
              {t('detail.layout.locked', { name: selected.name })}
            </p>
          )}
          {detaching && (
            <p className='mt-1.5 text-xs text-fk-amber'>
              {t('detail.layout.detaching', { name: detaching.name })}
            </p>
          )}
        </div>
      </FieldRow>
    </Section>
  )
}
