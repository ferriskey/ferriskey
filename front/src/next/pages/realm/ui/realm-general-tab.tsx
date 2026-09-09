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
  return (
    <>
      <Section title='Realm identity'>
        <FieldRow
          label='Realm name'
          description='Immutable after creation — it appears in every URL of this realm.'
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
          label='Display name'
          description='Human-readable name shown in the console and the login screens. Leave empty to use the realm name.'
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
          label='Default signing algorithm'
          description='Algorithm used to sign the tokens issued by this realm.'
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
              Read-only: the console never sends this setting, and the OpenID discovery
              document of this realm announces one algorithm only.
            </p>
          </div>
        </FieldRow>
      </Section>

      <DangerZone
        label='Delete this realm'
        description={`Once deleted, every client, account, role and session of this realm is permanently removed.${
          isMaster ? ' The master realm cannot be deleted.' : ''
        }`}
        buttonLabel='Delete realm'
        confirmTitle='Delete realm'
        confirmDescription={`This will permanently delete the realm "${realmName}" and all its data including users, clients, and roles.`}
        confirmText={realmName}
        disabled={isMaster}
        onConfirm={onDelete}
      />
    </>
  )
}
