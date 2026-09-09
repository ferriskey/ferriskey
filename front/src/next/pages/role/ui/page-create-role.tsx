import { ArrowLeft, Building2, Globe } from 'lucide-react'
import { Button } from '@/components/kit/button'
import { Input } from '@/components/ui/input'
import { Textarea } from '@/components/ui/textarea'
import SaveBar from '@/components/kit/save-bar'
import { ChoiceCards, FieldRow, Section, type Choice } from '@/components/kit'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'
import { Schemas } from '@/api/api.client'
import RoleClientPicker from './role-client-picker'
import RolePermissionsTab from './role-permissions-tab'

import Client = Schemas.Client

export type RoleScope = 'realm' | 'client'

export interface PageCreateRoleProps {
  clients: Client[]
  name: string
  description: string
  scope: RoleScope
  clientId?: string
  permissions: string[]
  errors: { name?: string; clientId?: string }
  canSubmit: boolean
  onNameChange: (v: string) => void
  onDescriptionChange: (v: string) => void
  onScopeChange: (v: RoleScope) => void
  onClientIdChange: (v: string) => void
  onPermissionsChange: (next: string[]) => void
  onBack: () => void
  onSubmit: () => void
}

const scopeOptions: Choice<RoleScope>[] = [
  {
    value: 'realm',
    label: 'Realm role',
    description: 'Grantable to any account of the realm.',
    icon: Globe,
  },
  {
    value: 'client',
    label: 'Client role',
    description: 'Only meaningful for the client it is attached to.',
    icon: Building2,
  },
]

export default function PageCreateRole({
  clients,
  name,
  description,
  scope,
  clientId,
  permissions,
  errors,
  canSubmit,
  onNameChange,
  onDescriptionChange,
  onScopeChange,
  onClientIdChange,
  onPermissionsChange,
  onBack,
  onSubmit,
}: PageCreateRoleProps) {
  return (
    <div className={cn('mx-auto', tokens.page.maxWidth, tokens.page.padding)}>
      <Button variant='ghost' size='sm' className='-ml-2 mb-2 text-neutral-500' onClick={onBack}>
        <ArrowLeft className='size-3.5' />
        Roles
      </Button>

      <div className='pb-3'>
        <h1 className={tokens.header.title}>New role</h1>
        <p className='mt-0.5 text-sm text-neutral-500'>
          A role bundles the permissions you grant to accounts and clients.
        </p>
      </div>

      <div className={tokens.page.blockGap}>
        <Section title='Definition'>
          <FieldRow
            label='Role name'
            description='Referenced as-is in the tokens issued for this realm.'
            htmlFor='new-role-name'
          >
            <Input
              id='new-role-name'
              value={name}
              onChange={(e) => onNameChange(e.target.value)}
              className='max-w-sm'
              aria-invalid={Boolean(errors.name)}
            />
            {errors.name && <p className='mt-1.5 text-fk-danger'>{errors.name}</p>}
          </FieldRow>

          <FieldRow
            label='Description'
            description='Helps administrators understand how far this role reaches.'
            htmlFor='new-role-description'
          >
            <Textarea
              id='new-role-description'
              value={description}
              onChange={(e) => onDescriptionChange(e.target.value)}
              className='max-w-lg'
              rows={3}
            />
          </FieldRow>

          <FieldRow
            label='Scope'
            description='Set at creation and final: a role cannot move between scopes afterwards.'
          >
            <ChoiceCards
              label='Role scope'
              value={scope}
              onChange={onScopeChange}
              options={scopeOptions}
            />
          </FieldRow>

          {scope === 'client' && (
            <FieldRow
              label='Client'
              description='The client this role is attached to.'
            >
              <RoleClientPicker
                clients={clients}
                value={clientId}
                onChange={onClientIdChange}
              />
              {errors.clientId && (
                <p className='mt-1.5 text-xs text-fk-danger'>{errors.clientId}</p>
              )}
            </FieldRow>
          )}
        </Section>

        <RolePermissionsTab value={permissions} onChange={onPermissionsChange} />
      </div>

      <SaveBar
        show={canSubmit}
        title='Create role'
        description='The role is created with the permissions selected above.'
        onCancel={onBack}
        actions={[{ label: 'Create role', onClick: onSubmit }]}
      />
    </div>
  )
}
