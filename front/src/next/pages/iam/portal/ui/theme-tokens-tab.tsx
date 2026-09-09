import { Input } from '@/components/ui/input'
import { ScrollArea } from '@/components/ui/scroll-area'
import { FieldRow, Section } from '@/components/kit'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'
import { ButtonsPanel } from '@/pages/portal-theme/components/panels/buttons-panel'
import { InputsPanel } from '@/pages/portal-theme/components/panels/inputs-panel'
import { WidgetPanel } from '@/pages/portal-theme/components/panels/widget-panel'
import { TypographyPanel } from '@/pages/portal-theme/components/panels/typography-panel'
import { PagePanel } from '@/pages/portal-theme/components/panels/page-panel'
import { PreviewCard } from '@/pages/portal-theme/components/preview-card'

export interface ThemeTokensTabProps {
  name: string
  onNameChange: (value: string) => void
  nameError?: string
}

export default function ThemeTokensTab({ name, onNameChange, nameError }: ThemeTokensTabProps) {
  return (
    <>
      <Section title='Identity'>
        <FieldRow
          label='Theme name'
          description='Names the theme in the console; end users never read it.'
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
        title='Design tokens'
        description='Every change is reflected live in the preview. Untouched fields keep the value the domain defaults to.'
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
