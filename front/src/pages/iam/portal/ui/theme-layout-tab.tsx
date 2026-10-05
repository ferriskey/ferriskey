import { useTranslation } from 'react-i18next'
import { FieldRow, RelationSelect, Section } from '@/components/kit'
import { portalLayoutRelationSource } from '@/api/portal-layout.relation'

export const NO_LAYOUT = '__none__'

export interface ThemeLayoutTabProps {
  layoutId: string
  selectedName?: string
  detachingName?: string
  onLayoutChange: (value: string) => void
}

export default function ThemeLayoutTab({
  layoutId,
  selectedName,
  detachingName,
  onLayoutChange,
}: ThemeLayoutTabProps) {
  const { t } = useTranslation('portal')

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
          <RelationSelect
            source={portalLayoutRelationSource}
            value={layoutId === NO_LAYOUT ? undefined : layoutId}
            onChange={(id) => onLayoutChange(id ?? NO_LAYOUT)}
            label={t('detail.layout.field.label')}
            noneLabel={t('detail.layout.option_none')}
          />

          {selectedName && (
            <p className='mt-1.5 text-xs text-neutral-400 dark:text-neutral-500'>
              {t('detail.layout.locked', { name: selectedName })}
            </p>
          )}
          {detachingName && (
            <p className='mt-1.5 text-xs text-fk-amber'>
              {t('detail.layout.detaching', { name: detachingName })}
            </p>
          )}
        </div>
      </FieldRow>
    </Section>
  )
}
