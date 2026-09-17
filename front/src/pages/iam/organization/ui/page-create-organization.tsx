import { ArrowLeft } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/kit/button'
import { Input } from '@/components/ui/input'
import { Textarea } from '@/components/ui/textarea'
import SaveBar from '@/components/kit/save-bar'
import { FieldRow, PageShell, Section, SwitchField } from '@/components/kit'
import { tokens } from '@/styles/style-tokens'

export interface CreateOrganizationDraft {
  name: string
  alias: string
  enabled: boolean
  domain: string
  redirectUrl: string
  description: string
}

export interface PageCreateOrganizationProps {
  draft: CreateOrganizationDraft
  errors: { name?: string; alias?: string }
  canSubmit: boolean
  onChange: (patch: Partial<CreateOrganizationDraft>) => void
  onBack: () => void
  onSubmit: () => void
}

export default function PageCreateOrganization({
  draft,
  errors,
  canSubmit,
  onChange,
  onBack,
  onSubmit,
}: PageCreateOrganizationProps) {
  const { t } = useTranslation('organization')

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
        <Section title={t('form.sections.definition')}>
          <FieldRow
            label={t('form.name.label')}
            description={t('form.name.description')}
            htmlFor='new-organization-name'
          >
            <Input
              id='new-organization-name'
              value={draft.name}
              onChange={(e) => onChange({ name: e.target.value })}
              className='max-w-sm'
              aria-invalid={Boolean(errors.name)}
            />
            {errors.name && <p className='mt-1.5 text-fk-danger text-xs'>{errors.name}</p>}
          </FieldRow>

          <FieldRow
            label={t('form.alias.label')}
            description={t('form.alias.description.create')}
            htmlFor='new-organization-alias'
          >
            <Input
              id='new-organization-alias'
              value={draft.alias}
              onChange={(e) => onChange({ alias: e.target.value })}
              className='max-w-sm'
              aria-invalid={Boolean(errors.alias)}
            />
            {errors.alias && <p className='mt-1.5 text-fk-danger text-xs'>{errors.alias}</p>}
          </FieldRow>

          <FieldRow
            label={t('form.enabled.label')}
            description={t('form.enabled.description.create')}
            htmlFor='new-organization-enabled'
          >
            <SwitchField
              id='new-organization-enabled'
              checked={draft.enabled}
              onCheckedChange={(enabled) => onChange({ enabled })}
            />
          </FieldRow>
        </Section>

        <Section title={t('form.sections.routing.title')}>
          <FieldRow
            label={t('form.domain.label')}
            description={t('form.domain.description.create')}
            htmlFor='new-organization-domain'
          >
            <Input
              id='new-organization-domain'
              value={draft.domain}
              onChange={(e) => onChange({ domain: e.target.value })}
              className='max-w-sm'
            />
          </FieldRow>

          <FieldRow
            label={t('form.redirect_url.label')}
            description={t('form.redirect_url.description')}
            htmlFor='new-organization-redirect-url'
          >
            <Input
              id='new-organization-redirect-url'
              value={draft.redirectUrl}
              onChange={(e) => onChange({ redirectUrl: e.target.value })}
              className='max-w-lg'
            />
          </FieldRow>

          <FieldRow
            label={t('form.description.label')}
            description={t('form.description.description')}
            htmlFor='new-organization-description'
          >
            <Textarea
              id='new-organization-description'
              value={draft.description}
              onChange={(e) => onChange({ description: e.target.value })}
              className='max-w-lg'
              rows={3}
            />
          </FieldRow>
        </Section>
      </div>

      <SaveBar
        show={canSubmit}
        title={t('create.save_bar.title')}
        description={t('create.save_bar.description')}
        onCancel={onBack}
        actions={[{ label: t('create.save_bar.submit'), onClick: onSubmit }]}
      />
    </PageShell>
  )
}
