import { useState } from 'react'
import { Check, Pencil, Plus, Trash2, X } from 'lucide-react'
import { Button } from '@/components/kit/button'
import { Input } from '@/components/ui/input'
import { Section } from '@/components/kit'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'
import { Schemas } from '@/api/api.client'

import OrganizationAttribute = Schemas.OrganizationAttribute

export interface OrganizationAttributesTabProps {
  attributes: OrganizationAttribute[]
  isLoading: boolean
  onUpsert: (key: string, value: string) => void
  onDelete: (key: string) => void
}

function AttributeRow({
  attribute,
  onEdit,
  onDelete,
}: {
  attribute: OrganizationAttribute
  onEdit: (key: string, value: string) => void
  onDelete: (key: string) => void
}) {
  const [editing, setEditing] = useState(false)
  const [draft, setDraft] = useState(attribute.value)

  const save = () => {
    if (draft.trim()) onEdit(attribute.key, draft.trim())
    setEditing(false)
  }

  const cancel = () => {
    setDraft(attribute.value)
    setEditing(false)
  }

  return (
    <div className='flex items-center gap-4 px-3 py-1.5 text-[13px]'>
      <span className='w-48 shrink-0 truncate font-mono-ui text-xs text-neutral-900'>
        {attribute.key}
      </span>
      {editing ? (
        <Input
          autoFocus
          value={draft}
          onChange={(e) => setDraft(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === 'Enter') save()
            if (e.key === 'Escape') cancel()
          }}
          className='h-8 max-w-xs'
        />
      ) : (
        <span className='min-w-0 flex-1 truncate text-neutral-600'>{attribute.value}</span>
      )}
      <div className='ml-auto flex shrink-0 items-center gap-1'>
        {editing ? (
          <>
            <Button
              variant='ghost'
              size='icon'
              className='size-7 text-fk-success'
              aria-label='Save attribute'
              onClick={save}
            >
              <Check />
            </Button>
            <Button
              variant='ghost'
              size='icon'
              className='size-7 text-neutral-400'
              aria-label='Cancel'
              onClick={cancel}
            >
              <X />
            </Button>
          </>
        ) : (
          <>
            <Button
              variant='ghost'
              size='icon'
              className='size-7 text-neutral-400'
              aria-label={`Edit ${attribute.key}`}
              onClick={() => setEditing(true)}
            >
              <Pencil />
            </Button>
            <Button
              variant='ghost'
              size='icon'
              className='size-7 text-neutral-400 hover:text-fk-danger'
              aria-label={`Delete ${attribute.key}`}
              onClick={() => onDelete(attribute.key)}
            >
              <Trash2 />
            </Button>
          </>
        )}
      </div>
    </div>
  )
}

function AddAttributeRow({
  onAdd,
  onCancel,
}: {
  onAdd: (key: string, value: string) => void
  onCancel: () => void
}) {
  const [key, setKey] = useState('')
  const [value, setValue] = useState('')

  const canSave = key.trim().length > 0 && value.trim().length > 0

  const save = () => {
    if (canSave) onAdd(key.trim(), value.trim())
  }

  return (
    <div className='flex items-center gap-4 bg-fk-primary-soft/40 px-3 py-1.5'>
      <Input
        autoFocus
        placeholder='Key'
        value={key}
        onChange={(e) => setKey(e.target.value)}
        onKeyDown={(e) => e.key === 'Escape' && onCancel()}
        className='h-8 w-48 shrink-0 font-mono-ui text-xs'
      />
      <Input
        placeholder='Value'
        value={value}
        onChange={(e) => setValue(e.target.value)}
        onKeyDown={(e) => {
          if (e.key === 'Enter') save()
          if (e.key === 'Escape') onCancel()
        }}
        className='h-8 max-w-xs'
      />
      <div className='ml-auto flex shrink-0 items-center gap-1'>
        <Button
          variant='ghost'
          size='icon'
          className='size-7 text-fk-success'
          aria-label='Save attribute'
          disabled={!canSave}
          onClick={save}
        >
          <Check />
        </Button>
        <Button
          variant='ghost'
          size='icon'
          className='size-7 text-neutral-400'
          aria-label='Cancel'
          onClick={onCancel}
        >
          <X />
        </Button>
      </div>
    </div>
  )
}

export default function OrganizationAttributesTab({
  attributes,
  isLoading,
  onUpsert,
  onDelete,
}: OrganizationAttributesTabProps) {
  const [adding, setAdding] = useState(false)

  return (
    <Section
      title={`Attributes (${attributes.length})`}
      description='Custom key-value metadata carried by this organization.'
      contained={false}
      action={
        !adding && (
          <Button size='sm' variant='outline' onClick={() => setAdding(true)}>
            <Plus /> Add attribute
          </Button>
        )
      }
    >
      <div className={cn(tokens.surface.panel, tokens.surface.divider)}>
        <div
          className={cn(
            'flex items-center gap-4 bg-neutral-50/60 px-3 py-1.5',
            tokens.table.headerText
          )}
        >
          <span className='w-48 shrink-0'>Key</span>
          <span>Value</span>
        </div>

        {adding && (
          <AddAttributeRow
            onAdd={(key, value) => {
              onUpsert(key, value)
              setAdding(false)
            }}
            onCancel={() => setAdding(false)}
          />
        )}

        {isLoading ? (
          Array.from({ length: 4 }).map((_, i) => (
            <div key={i} className='flex items-center gap-4 px-3 py-2.5'>
              <div className='h-3 w-40 animate-pulse rounded bg-neutral-100' />
              <div className='h-3 w-56 animate-pulse rounded bg-neutral-100' />
            </div>
          ))
        ) : attributes.length === 0 && !adding ? (
          <p className='px-3 py-8 text-center text-sm text-neutral-500'>
            No attribute defined for this organization.
          </p>
        ) : (
          attributes.map((attribute) => (
            <AttributeRow
              key={attribute.id}
              attribute={attribute}
              onEdit={onUpsert}
              onDelete={onDelete}
            />
          ))
        )}
      </div>
    </Section>
  )
}
