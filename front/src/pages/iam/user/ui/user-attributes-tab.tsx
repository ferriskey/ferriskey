import { useState } from 'react'
import { Check, Pencil, Plus, Trash2, X } from 'lucide-react'
import { useTranslation } from 'react-i18next'
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
  const { t } = useTranslation('user')
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
      <span className='w-48 shrink-0 truncate font-mono-ui text-sm text-neutral-900 dark:text-neutral-100'>
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
        <span className='min-w-0 flex-1 truncate font-mono-ui text-xs text-neutral-600 dark:text-neutral-400'>
          {attribute.value}
        </span>
      )}

      <div className='flex shrink-0 gap-0.5'>
        {editing ? (
          <>
            <Button
              variant='ghost'
              size='icon'
              aria-label={t('detail.attributes.list.save', { key: attribute.key })}
              onClick={save}
              className='size-7 text-fk-success'
            >
              <Check />
            </Button>
            <Button
              variant='ghost'
              size='icon'
              aria-label={t('detail.attributes.list.cancel', { key: attribute.key })}
              onClick={cancel}
              className='size-7 text-neutral-400 dark:text-neutral-500'
            >
              <X />
            </Button>
          </>
        ) : (
          <>
            <Button
              variant='ghost'
              size='icon'
              aria-label={t('detail.attributes.list.edit', { key: attribute.key })}
              onClick={() => setEditing(true)}
              className='size-7 text-neutral-400 hover:text-neutral-900 dark:text-neutral-500 dark:hover:text-neutral-100'
            >
              <Pencil />
            </Button>
            <Button
              variant='ghost'
              size='icon'
              aria-label={t('detail.attributes.list.delete', { key: attribute.key })}
              onClick={() => onDelete(attribute.key)}
              className='size-7 text-neutral-400 dark:text-neutral-500 hover:text-fk-danger'
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
  const { t } = useTranslation('user')
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
        title={t('detail.attributes.list.title')}
        description={
          attributes.length > 0
            ? t('detail.attributes.list.description', { count: attributes.length })
            : t('detail.attributes.list.empty_description')
        }
        contained={!isLoading && attributes.length > 0}
      >
        {isLoading ? (
          <div className='space-y-2'>
            {Array.from({ length: 3 }).map((_, i) => (
              <div key={i} className='h-8 animate-pulse rounded-md bg-neutral-100 dark:bg-fk-raised' />
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
          <p className='rounded-md border border-dashed border-fk-line px-4 py-3 text-sm text-neutral-500 dark:text-neutral-400'>
            {t('detail.attributes.list.empty')}
          </p>
        )}
      </Section>

      <Section
        title={t('detail.attributes.add.title')}
        description={t('detail.attributes.add.description')}
        contained={false}
      >
        <div className='flex flex-wrap items-start gap-2'>
          <div className='min-w-48'>
            <Input
              value={key}
              onChange={(e) => setKey(e.target.value)}
              placeholder={t('detail.attributes.add.key_placeholder')}
              className='h-8'
            />
            {duplicate && (
              <p className='mt-1 text-xs text-fk-amber'>
                {t('detail.attributes.add.duplicate')}
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
            placeholder={t('detail.attributes.add.value_placeholder')}
            className='h-8 min-w-48 flex-1 font-mono-ui text-xs'
          />
          <Button size='sm' onClick={add} disabled={!canAdd}>
            <Plus /> {t('detail.attributes.add.submit')}
          </Button>
        </div>
      </Section>
    </>
  )
}
