import { useState } from 'react'
import { Check, Pencil, Plus, Trash2, X } from 'lucide-react'
import { Button } from '@/components/kit/button'
import { Input } from '@/components/ui/input'
import { Section } from '@/components/kit'
import { tokens } from '@/styles/style-tokens'
import { Schemas } from '@/api/api.client'

import UserAttribute = Schemas.UserAttribute

export interface UserAttributesTabProps {
  attributes: UserAttribute[]
  isLoading: boolean
  onUpsert: (key: string, value: string) => void
  onDelete: (key: string) => void
}

interface AttributeRowProps {
  attribute: UserAttribute
  onEdit: (key: string, value: string) => void
  onDelete: (key: string) => void
}

function AttributeRow({ attribute, onEdit, onDelete }: AttributeRowProps) {
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
    <li className='flex items-center gap-3 py-2'>
      <span className='w-48 shrink-0 truncate font-mono-ui text-sm text-neutral-900'>
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
          className='h-8 min-w-0 flex-1 font-mono-ui text-xs'
        />
      ) : (
        <span className='min-w-0 flex-1 truncate font-mono-ui text-xs text-neutral-600'>
          {attribute.value}
        </span>
      )}

      <div className='flex shrink-0 gap-0.5'>
        {editing ? (
          <>
            <Button
              variant='ghost'
              size='icon'
              aria-label={`Save ${attribute.key}`}
              onClick={save}
              className='size-7 text-fk-success'
            >
              <Check />
            </Button>
            <Button
              variant='ghost'
              size='icon'
              aria-label={`Cancel editing ${attribute.key}`}
              onClick={cancel}
              className='size-7 text-neutral-400'
            >
              <X />
            </Button>
          </>
        ) : (
          <>
            <Button
              variant='ghost'
              size='icon'
              aria-label={`Edit ${attribute.key}`}
              onClick={() => setEditing(true)}
              className='size-7 text-neutral-400 hover:text-neutral-900'
            >
              <Pencil />
            </Button>
            <Button
              variant='ghost'
              size='icon'
              aria-label={`Delete ${attribute.key}`}
              onClick={() => onDelete(attribute.key)}
              className='size-7 text-neutral-400 hover:text-fk-danger'
            >
              <Trash2 />
            </Button>
          </>
        )}
      </div>
    </li>
  )
}

export default function UserAttributesTab({
  attributes,
  isLoading,
  onUpsert,
  onDelete,
}: UserAttributesTabProps) {
  const [key, setKey] = useState('')
  const [value, setValue] = useState('')

  const duplicate = attributes.some((a) => a.key === key.trim())
  const canAdd = key.trim().length > 0 && value.trim().length > 0

  const add = () => {
    if (!canAdd) return
    onUpsert(key.trim(), value.trim())
    setKey('')
    setValue('')
  }

  return (
    <>
      <Section
        title='Custom attributes'
        description={
          attributes.length > 0
            ? `${attributes.length} attribute${attributes.length > 1 ? 's' : ''} on this account.`
            : 'Key-value metadata carried by this account.'
        }
        contained={!isLoading && attributes.length > 0}
      >
        {isLoading ? (
          <div className='space-y-2'>
            {Array.from({ length: 3 }).map((_, i) => (
              <div key={i} className='h-8 animate-pulse rounded-md bg-neutral-100' />
            ))}
          </div>
        ) : attributes.length > 0 ? (
          <ul className={tokens.surface.divider}>
            {attributes.map((attribute) => (
              <AttributeRow
                key={attribute.id}
                attribute={attribute}
                onEdit={onUpsert}
                onDelete={onDelete}
              />
            ))}
          </ul>
        ) : (
          <p className='rounded-md border border-dashed border-fk-line px-4 py-3 text-sm text-neutral-500'>
            No attribute defined for this user.
          </p>
        )}
      </Section>

      <Section
        title='Add an attribute'
        description='Attributes reach the tokens through the mappers of a client scope.'
        contained={false}
      >
        <div className='flex flex-wrap items-start gap-2'>
          <div className='min-w-48'>
            <Input
              value={key}
              onChange={(e) => setKey(e.target.value)}
              placeholder='key'
              className='h-8 font-mono-ui text-xs'
            />
            {duplicate && (
              <p className='mt-1 text-xs text-fk-amber'>
                This key already exists — adding it replaces its value.
              </p>
            )}
          </div>
          <Input
            value={value}
            onChange={(e) => setValue(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === 'Enter') add()
              if (e.key === 'Escape') {
                setKey('')
                setValue('')
              }
            }}
            placeholder='value'
            className='h-8 min-w-48 flex-1 font-mono-ui text-xs'
          />
          <Button size='sm' onClick={add} disabled={!canAdd}>
            <Plus /> Add
          </Button>
        </div>
      </Section>
    </>
  )
}
