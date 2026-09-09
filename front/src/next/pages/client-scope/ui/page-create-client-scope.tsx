import { ArrowLeft } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Textarea } from '@/components/ui/textarea'
import FloatingActionBar from '@/components/ui/floating-action-bar'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { FieldRow, Section } from '@/components/kit'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'
import ScopeTypeHint from './scope-type-hint'

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
  return (
    <div className={cn('mx-auto', tokens.page.maxWidth, tokens.page.padding)}>
      <Button variant='ghost' size='sm' className='-ml-2 mb-2 text-neutral-500' onClick={onBack}>
        <ArrowLeft className='size-3.5' />
        Client Scopes
      </Button>

      <div className='pb-3'>
        <h1 className={tokens.header.title}>New client scope</h1>
        <p className='mt-0.5 text-sm text-neutral-500'>
          A client scope groups the claims a client can ask for in its tokens.
        </p>
      </div>

      <div className={tokens.page.blockGap}>
        <Section
          title='Client Scope Details'
          description='Identity of the scope, and how clients get it.'
        >
          <FieldRow
            label='Name'
            description='Requested as-is by a client in the scope parameter of an authorization request.'
            htmlFor='new-scope-name'
          >
            <Input
              id='new-scope-name'
              value={name}
              onChange={(e) => onNameChange(e.target.value)}
              className='max-w-sm font-mono-ui'
              aria-invalid={Boolean(nameError)}
            />
            {nameError && <p className='mt-1.5 text-xs text-fk-danger'>{nameError}</p>}
          </FieldRow>

          <FieldRow
            label='Description'
            description='Read by administrators only — it never reaches an issued token.'
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
            label='Protocol'
            description='Fixed at creation: only OpenID Connect is supported today, and it decides which protocol mappers this scope can carry.'
            htmlFor='new-scope-protocol'
          >
            <Input
              id='new-scope-protocol'
              value={protocol}
              disabled
              className='max-w-sm font-mono-ui'
            />
          </FieldRow>

          <FieldRow
            label='Type'
            description='Decides how this scope is attached to the clients of the realm.'
            htmlFor='new-scope-type'
          >
            <div className='max-w-sm'>
              <Select value={scopeType} onValueChange={(v) => onScopeTypeChange(v as ScopeTypeChoice)}>
                <SelectTrigger id='new-scope-type' className='w-full'>
                  <SelectValue placeholder='Select type' />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value='optional'>Optional</SelectItem>
                  <SelectItem value='default'>Default</SelectItem>
                </SelectContent>
              </Select>
              <ScopeTypeHint scopeType={scopeType} />
            </div>
          </FieldRow>
        </Section>
      </div>

      <FloatingActionBar
        show={canSubmit}
        title='Create client scope'
        description='The scope is created empty — its protocol mappers are added afterwards.'
        onCancel={onBack}
        actions={[{ label: isPending ? 'Creating…' : 'Create', onClick: onSubmit }]}
      />
    </div>
  )
}
