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
  const meta = applicationTypeMeta(type)

  return (
    <Section
      title='Client credentials'
      description='What this application presents to prove who it is.'
    >
      <FieldRow
        label='Client ID'
        description='Public: it travels in the browser on every sign-in. It is not a credential.'
      >
        <code className='flex h-9 max-w-lg items-center overflow-x-auto rounded-md border border-fk-line bg-neutral-50 px-2.5 font-mono-ui text-xs text-neutral-700 dark:bg-fk-surface dark:text-neutral-300'>
          {clientId}
        </code>
      </FieldRow>

      <FieldRow
        label='Client secret'
        description={`This application has no secret. It runs where your users can read its code, so a secret would leak — the sign-in is protected by ${meta.flow} instead.`}
      >
        <p className='text-xs text-neutral-500 dark:text-neutral-400'>
          Nothing to copy here. Never paste a secret from another application into this one.
        </p>
      </FieldRow>
    </Section>
  )
}
