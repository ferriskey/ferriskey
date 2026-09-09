import { Input } from '@/components/ui/input'
import { Textarea } from '@/components/ui/textarea'
import { FieldRow, Pill, Section } from '@/components/kit'
import { DangerZone } from '@/components/danger-zone'
import { Schemas } from '@/api/api.client'

import Role = Schemas.Role

export interface RoleSettingsTabProps {
  role: Role
  name: string
  description: string
  nameError?: string
  onNameChange: (v: string) => void
  onDescriptionChange: (v: string) => void
  onDelete: () => void
}

export default function RoleSettingsTab({
  role,
  name,
  description,
  nameError,
  onNameChange,
  onDescriptionChange,
  onDelete,
}: RoleSettingsTabProps) {
  return (
    <>
      <Section title='Definition'>
        <FieldRow
          label='Role name'
          description='Referenced as-is in the tokens issued for this realm.'
          htmlFor='role-name'
        >
          <Input
            id='role-name'
            value={name}
            onChange={(e) => onNameChange(e.target.value)}
            className='max-w-sm font-mono-ui text-xs'
            aria-invalid={Boolean(nameError)}
          />
          {nameError && <p className='mt-1.5 text-xs text-fk-danger'>{nameError}</p>}
        </FieldRow>

        <FieldRow
          label='Description'
          description='Helps administrators understand how far this role reaches.'
          htmlFor='role-description'
        >
          <Textarea
            id='role-description'
            value={description}
            onChange={(e) => onDescriptionChange(e.target.value)}
            className='max-w-lg'
            rows={3}
          />
        </FieldRow>

        <FieldRow
          label='Scope'
          description={
            role.client_id
              ? 'Attached to a client at creation. Moving a role between scopes is not supported.'
              : 'Shared across the realm. Moving a role between scopes is not supported.'
          }
        >
          <div className='flex items-center gap-2'>
            <Pill tone={role.client_id ? 'violet' : 'info'} mono>
              {role.client_id ? 'client' : 'realm'}
            </Pill>
            {role.client?.client_id && (
              <Pill mono>{role.client.client_id}</Pill>
            )}
          </div>
        </FieldRow>
      </Section>

      <DangerZone
        label='Delete this role'
        description='Once deleted, all user assignments for this role will be permanently removed. Tokens already issued stay valid until they expire.'
        buttonLabel='Delete role'
        confirmTitle='Delete role'
        confirmDescription={`This will permanently delete the role "${role.name}" and remove all user assignments.`}
        onConfirm={onDelete}
      />
    </>
  )
}
