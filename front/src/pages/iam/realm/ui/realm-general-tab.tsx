import { useTranslation } from 'react-i18next'
import { Input } from '@/components/ui/input'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { FieldRow, Section } from '@/components/kit'
import { DangerZone } from '@/components/kit/danger-zone'
import { REALM_NAMESPACE } from '../realm-namespace'

export interface RealmGeneralTabProps {
  realmName: string
  displayName: string
  displayNameError?: string
  signingAlgorithm: string
  isMaster: boolean
  onDisplayNameChange: (v: string) => void
  onDelete: () => void
}

export default function RealmGeneralTab({
  realmName,
  displayName,
  displayNameError,
  signingAlgorithm,
  isMaster,
  onDisplayNameChange,
  onDelete,
}: RealmGeneralTabProps) {
  const { t } = useTranslation(REALM_NAMESPACE)

  return (
    <>
      <Section title={t('general.identity.title')}>
        <FieldRow
          label={t('general.name.label')}
          description={t('general.name.description')}
          htmlFor='realm-name'
        >
          <Input
            id='realm-name'
            value={realmName}
            disabled
            className='max-w-sm'
          />
        </FieldRow>

        <FieldRow
          label={t('general.display_name.label')}
          description={t('general.display_name.description')}
          htmlFor='realm-display-name'
        >
          <Input
            id='realm-display-name'
            value={displayName}
            onChange={(e) => onDisplayNameChange(e.target.value)}
            className='max-w-sm'
            aria-invalid={Boolean(displayNameError)}
          />
          {displayNameError && (
            <p className='mt-1.5 text-xs text-fk-danger'>{displayNameError}</p>
          )}
        </FieldRow>

        <FieldRow
          label={t('general.signing_algorithm.label')}
          description={t('general.signing_algorithm.description')}
        >
          <div className='max-w-sm'>
            <Select value={signingAlgorithm} disabled>
              <SelectTrigger className='w-full'>
                <SelectValue />
              </SelectTrigger>
              <SelectContent position='popper'>
                <SelectItem value={signingAlgorithm}>{signingAlgorithm}</SelectItem>
              </SelectContent>
            </Select>
            <p className='mt-1.5 text-xs text-neutral-500 dark:text-neutral-400'>
              {t('general.signing_algorithm.note')}
            </p>
          </div>
        </FieldRow>
      </Section>

      <DangerZone
        label={t('general.danger.label')}
        description={
          isMaster
            ? t('general.danger.description_master')
            : t('general.danger.description')
        }
        buttonLabel={t('general.danger.button')}
        confirmTitle={t('general.danger.confirm_title')}
        confirmDescription={t('general.danger.confirm_description', { name: realmName })}
        confirmText={realmName}
        disabled={isMaster}
        onConfirm={onDelete}
      />
    </>
  )
}
