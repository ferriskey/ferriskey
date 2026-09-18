import { useState } from 'react'
import { Plus, Trash2, X } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/kit/button'
import { Input } from '@/components/ui/input'

const HEADER_NAME_EXAMPLE = 'X-Api-Key'

export interface WebhookHeader {
  key: string
  value: string
}

export interface WebhookHeadersFieldProps {
  headers: WebhookHeader[]
  onChange: (next: WebhookHeader[]) => void
}

export default function WebhookHeadersField({
  headers,
  onChange,
}: WebhookHeadersFieldProps) {
  const { t } = useTranslation('webhook')
  const [adding, setAdding] = useState(false)
  const [draftKey, setDraftKey] = useState('')
  const [draftValue, setDraftValue] = useState('')

  const commit = () => {
    if (!draftKey || !draftValue) return
    onChange([...headers, { key: draftKey, value: draftValue }])
    setDraftKey('')
    setDraftValue('')
    setAdding(false)
  }

  const cancel = () => {
    setDraftKey('')
    setDraftValue('')
    setAdding(false)
  }

  const update = (index: number, field: 'key' | 'value', value: string) =>
    onChange(headers.map((h, i) => (i === index ? { ...h, [field]: value } : h)))

  const updateKey = (index: number, value: string) => update(index, 'key', value)

  const updateValue = (index: number, value: string) => update(index, 'value', value)

  return (
    <div className='max-w-xl space-y-2'>
      {headers.length === 0 && !adding && (
        <p className='text-xs text-neutral-500 dark:text-neutral-400'>{t('headers.empty')}</p>
      )}

      {headers.map((header, index) => (
        <div key={index} className='flex items-center gap-2'>
          <Input
            value={header.key}
            onChange={(e) => updateKey(index, e.target.value)}
            aria-label={t('headers.name_label')}
            className='flex-1'
          />
          <Input
            value={header.value}
            onChange={(e) => updateValue(index, e.target.value)}
            aria-label={t('headers.value_label')}
            className='flex-1'
          />
          <Button
            variant='ghost'
            size='icon'
            aria-label={t('headers.remove', { name: header.key })}
            onClick={() => onChange(headers.filter((_, i) => i !== index))}
            className='size-8 text-neutral-400 dark:text-neutral-500 hover:text-fk-danger'
          >
            <Trash2 />
          </Button>
        </div>
      ))}

      {adding ? (
        <div className='flex items-center gap-2'>
          <Input
            value={draftKey}
            onChange={(e) => setDraftKey(e.target.value)}
            placeholder={HEADER_NAME_EXAMPLE}
            aria-label={t('headers.new_name_label')}
            className='flex-1'
          />
          <Input
            value={draftValue}
            onChange={(e) => setDraftValue(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === 'Enter') {
                e.preventDefault()
                commit()
              }
            }}
            placeholder={t('headers.value_placeholder')}
            aria-label={t('headers.new_value_label')}
            className='flex-1 font-mono-ui text-xs'
          />
          <Button size='sm' onClick={commit} disabled={!draftKey || !draftValue}>
            {t('headers.add')}
          </Button>
          <Button
            variant='ghost'
            size='icon'
            aria-label={t('headers.cancel')}
            onClick={cancel}
            className='size-8 text-neutral-400 dark:text-neutral-500'
          >
            <X />
          </Button>
        </div>
      ) : (
        <Button variant='outline' size='sm' onClick={() => setAdding(true)}>
          <Plus /> {t('headers.add_header')}
        </Button>
      )}
    </div>
  )
}
