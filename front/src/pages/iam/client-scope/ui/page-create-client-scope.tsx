import { ArrowLeft } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/kit/button'
import { Input } from '@/components/ui/input'
import { Textarea } from '@/components/ui/textarea'
import SaveBar from '@/components/kit/save-bar'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { FieldRow, PageShell, Section } from '@/components/kit'
import { tokens } from '@/styles/style-tokens'
import ScopeTypeHint from './scope-type-hint'
import { SCOPE_TYPE_CHOICES } from '../scope-type'

export type ScopeTypeChoice = 'optional' | 'default'

export interface PageCreateClientScopeProps {
  name: string
  description: string
  protocol: string
  scopeType: ScopeTypeChoice
  nameError?: string
  canSubmit: boolean
  isPending: boolean
  onNameChange: (v: string) => void
  onDescriptionChange: (v: string) => void
  onScopeTypeChange: (v: ScopeTypeChoice) => void
  onBack: () => void
  onSubmit: () => void
}

export default function PageCreateClientScope({
  name,
  description,
  protocol,
  scopeType,
  nameError,
  canSubmit,
  isPending,
  onNameChange,
  onDescriptionChange,
  onScopeTypeChange,
  onBack,
  onSubmit,
}: PageCreateClientScopeProps) {
  const { t } = useTranslation('client-scope')

  return (
    <PageShell>
      <Button variant='ghost' size='sm' className='-ml-2 mb-2 text-neutral-500 dark:text-neutral-400' onClick={onBack}>
        <ArrowLeft className='size-3.5' />
        {t('create.back')}
      </Button>

      <div className='pb-3'>
        <h1 className={tokens.header.title}>{t('create.title')}</h1>
        <p className='mt-0.5 text-sm text-neutral-500 dark:text-neutral-400'>
          {t('create.subtitle')}
        </p>
      </div>

      <div className={tokens.page.blockGap}>
        <Section
          title={t('create.section.title')}
          description={t('create.section.description')}
        >
          <FieldRow
            label={t('scope_form.name.label')}
            description={t('scope_form.name.description')}
            htmlFor='new-scope-name'
          >
            <Input
              id='new-scope-name'
              value={name}
              onChange={(e) => onNameChange(e.target.value)}
              className='max-w-sm'
              aria-invalid={Boolean(nameError)}
            />
            {nameError && <p className='mt-1.5 text-fk-danger text-xs'>{nameError}</p>}
          </FieldRow>

          <FieldRow
            label={t('scope_form.description.label')}
            description={t('scope_form.description.description')}
            htmlFor='new-scope-description'
          >
            <Textarea
              id='new-scope-description'
              value={description}
              onChange={(e) => onDescriptionChange(e.target.value)}
              className='max-w-lg'
              rows={3}
            />
          </FieldRow>

          <FieldRow
            label={t('scope_form.protocol.label')}
            description={t('scope_form.protocol.description')}
            htmlFor='new-scope-protocol'
          >
            <Input
              id='new-scope-protocol'
              value={protocol}
              disabled
              className='max-w-sm'
            />
          </FieldRow>

          <FieldRow
            label={t('scope_form.type.label')}
            description={t('scope_form.type.description')}
            htmlFor='new-scope-type'
          >
            <div className='max-w-sm'>
              <Select value={scopeType} onValueChange={(v) => onScopeTypeChange(v as ScopeTypeChoice)}>
                <SelectTrigger id='new-scope-type' className='w-full'>
                  <SelectValue placeholder={t('scope_form.type.placeholder')} />
                </SelectTrigger>
                <SelectContent>
                  {SCOPE_TYPE_CHOICES.map((choice) => (
                    <SelectItem key={choice.value} value={choice.value}>
                      {t(choice.labelKey)}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
              <ScopeTypeHint scopeType={scopeType} />
            </div>
          </FieldRow>
        </Section>
      </div>

      <SaveBar
        show={canSubmit}
        title={t('create.save_bar.title')}
        description={t('create.save_bar.description')}
        onCancel={onBack}
        actions={[
          {
            label: isPending ? t('create.save_bar.submitting') : t('create.save_bar.submit'),
            onClick: onSubmit,
          },
        ]}
      />
    </PageShell>
  )
}
