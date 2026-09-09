import { Input } from '@/components/ui/input'
import { Textarea } from '@/components/ui/textarea'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { FieldRow, Section } from '@/components/kit'
import { DangerZone } from '@/components/kit/danger-zone'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'
import { Schemas } from '@/api/api.client'
import ScopeTypeHint from './scope-type-hint'
import type { ScopeTypeChoice } from './page-create-client-scope'

import ClientScope = Schemas.ClientScope

export interface ClientScopeSettingsTabProps {
  scope: ClientScope
  name: string
  description: string
  scopeType: ScopeTypeChoice
  nameError?: string
  onNameChange: (v: string) => void
  onDescriptionChange: (v: string) => void
  onScopeTypeChange: (v: ScopeTypeChoice) => void
  onDelete: () => void
}

export default function ClientScopeSettingsTab({
  scope,
  name,
  description,
  scopeType,
  nameError,
  onNameChange,
  onDescriptionChange,
  onScopeTypeChange,
  onDelete,
}: ClientScopeSettingsTabProps) {
  const attributes = scope.attributes ?? []

  return (
    <>
      <Section
        title='General Information'
        description='Identity of the scope, and how clients get it.'
      >
        <FieldRow
          label='Name'
          description='Requested as-is by a client in the scope parameter of an authorization request.'
          htmlFor='scope-name'
        >
          <Input
            id='scope-name'
            value={name}
            onChange={(e) => onNameChange(e.target.value)}
            className='max-w-sm'
            aria-invalid={Boolean(nameError)}
          />
          {nameError && <p className='mt-1.5 text-fk-danger'>{nameError}</p>}
        </FieldRow>

        <FieldRow
          label='Description'
          description='Read by administrators only — it never reaches an issued token.'
          htmlFor='scope-description'
        >
          <Textarea
            id='scope-description'
            value={description}
            onChange={(e) => onDescriptionChange(e.target.value)}
            className='max-w-lg'
            rows={3}
          />
        </FieldRow>

        <FieldRow
          label='Protocol'
          description='Fixed at creation: only OpenID Connect is supported today, and it decides which protocol mappers this scope can carry.'
          htmlFor='scope-protocol'
        >
          <Input
            id='scope-protocol'
            value={scope.protocol}
            disabled
            className='max-w-sm'
          />
        </FieldRow>

        <FieldRow
          label='Type'
          description='Decides how this scope is attached to the clients of the realm.'
          htmlFor='scope-type'
        >
          <div className='max-w-sm'>
            <Select value={scopeType} onValueChange={(v) => onScopeTypeChange(v as ScopeTypeChoice)}>
              <SelectTrigger id='scope-type' className='w-full'>
                <SelectValue placeholder='Select type' />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value='optional'>Optional</SelectItem>
                <SelectItem value='default'>Default</SelectItem>
              </SelectContent>
            </Select>
            <ScopeTypeHint scopeType={scopeType} />
            {scope.default_scope_type === 'NONE' && (
              <p className='mt-1.5 text-xs text-fk-amber'>
                This scope is stored as none. The API only accepts optional or default, so
                saving turns it into the value selected above.
              </p>
            )}
          </div>
        </FieldRow>
      </Section>

      <Section
        title='Attributes'
        description='Attributes of this client scope, as the API returns them.'
        contained={attributes.length > 0}
      >
        {attributes.length > 0 ? (
          <dl className={tokens.surface.divider}>
            {attributes.map((attribute) => (
              <div
                key={attribute.id}
                className='grid gap-x-6 py-2.5 md:grid-cols-[minmax(0,18rem)_minmax(0,1fr)]'
              >
                <dt className='font-mono-ui text-xs text-neutral-500'>{attribute.name}</dt>
                <dd className='min-w-0 break-all font-mono-ui text-xs text-neutral-900'>
                  {attribute.value || '—'}
                </dd>
              </div>
            ))}
          </dl>
        ) : (
          <p
            className={cn(
              'rounded-sm border border-dashed border-fk-line px-4 py-3 text-xs text-neutral-500'
            )}
          >
            No attribute configured.
          </p>
        )}
      </Section>

      <DangerZone
        label='Delete this client scope'
        description='Once deleted, all associated protocol mappers and client mappings will be permanently removed.'
        buttonLabel='Delete scope'
        confirmTitle='Delete client scope'
        confirmDescription={`This will permanently delete the scope "${scope.name}" and all its associated protocol mappers and client mappings.`}
        onConfirm={onDelete}
      />
    </>
  )
}
