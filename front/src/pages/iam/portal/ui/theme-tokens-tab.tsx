import { useTranslation } from 'react-i18next'
import { Input } from '@/components/ui/input'
import { ScrollArea } from '@/components/ui/scroll-area'
import { FieldRow, Section } from '@/components/kit'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'
import { ButtonsPanel } from '@/pages/iam/portal/theme-builder/components/panels/buttons-panel'
import { InputsPanel } from '@/pages/iam/portal/theme-builder/components/panels/inputs-panel'
import { WidgetPanel } from '@/pages/iam/portal/theme-builder/components/panels/widget-panel'
import { TypographyPanel } from '@/pages/iam/portal/theme-builder/components/panels/typography-panel'
import { PagePanel } from '@/pages/iam/portal/theme-builder/components/panels/page-panel'
import { PreviewCard } from '@/pages/iam/portal/theme-builder/components/preview-card'

export interface ThemeTokensTabProps {
  name: string
  onNameChange: (value: string) => void
  nameError?: string
}

export default function ThemeTokensTab({ name, onNameChange, nameError }: ThemeTokensTabProps) {
  const { t } = useTranslation('portal')

  return (
    <>
      <Section title={t('detail.identity.title')}>
        <FieldRow
          label={t('detail.identity.name.label')}
          description={t('detail.identity.name.description')}
          htmlFor='portal-theme-name'
        >
          <Input
            id='portal-theme-name'
            value={name}
            onChange={(e) => onNameChange(e.target.value)}
            className='max-w-sm'
            aria-invalid={Boolean(nameError)}
          />
          {nameError && <p className='mt-1.5 text-fk-danger text-xs'>{nameError}</p>}
        </FieldRow>
      </Section>

      <Section
        title={t('detail.tokens.title')}
        description={t('detail.tokens.description')}
        contained={false}
      >
        <div
          className={cn(
            tokens.surface.panel,
            'grid h-[36rem] grid-cols-1 overflow-hidden lg:grid-cols-[1fr_22rem]'
          )}
        >
          <div className='min-h-0 overflow-hidden bg-neutral-50 dark:bg-fk-surface'>
            <PreviewCard />
          </div>
          <aside className='min-h-0 border-t border-fk-line lg:border-t-0 lg:border-l'>
            <ScrollArea className='h-full'>
              <div className='flex flex-col gap-6 p-4'>
                <ButtonsPanel />
                <InputsPanel />
                <WidgetPanel />
                <TypographyPanel />
                <PagePanel />
              </div>
            </ScrollArea>
          </aside>
        </div>
      </Section>
    </>
  )
}
