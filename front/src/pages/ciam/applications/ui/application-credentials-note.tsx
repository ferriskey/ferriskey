import { useTranslation } from 'react-i18next'
import { FieldRow, Section } from '@/components/kit'
import { applicationTypeMeta, type ApplicationType } from '../application-types'

export interface ApplicationCredentialsNoteProps {
  type: ApplicationType
  clientId: string
}

export default function ApplicationCredentialsNote({
  type,
  clientId,
}: ApplicationCredentialsNoteProps) {
  const { t } = useTranslation('console')
  const meta = applicationTypeMeta(type, t)

  return (
    <Section
      title={t('applications.credentials.title')}
      description={t('applications.credentials.description')}
    >
      <FieldRow
        label={t('applications.credentials.client_id_label')}
        description={t('applications.credentials.client_id_description')}
      >
        <code className='flex h-9 max-w-lg items-center overflow-x-auto rounded-md border border-fk-line bg-neutral-50 px-2.5 font-mono-ui text-xs text-neutral-700 dark:bg-fk-surface dark:text-neutral-300'>
          {clientId}
        </code>
      </FieldRow>

      <FieldRow
        label={t('applications.credentials.secret_label')}
        description={t('applications.credentials.secret_description', { flow: meta.flow })}
      >
        <p className='text-xs text-neutral-500 dark:text-neutral-400'>
          {t('applications.credentials.secret_note')}
        </p>
      </FieldRow>
    </Section>
  )
}
