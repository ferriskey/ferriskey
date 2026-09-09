import { useState } from 'react'
import { Plus, Trash2, X } from 'lucide-react'
import { Button } from '@/components/kit/button'
import { Input } from '@/components/ui/input'

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

  return (
    <div className='max-w-xl space-y-2'>
      {headers.length === 0 && !adding && (
        <p className='text-xs text-neutral-500 dark:text-neutral-400'>
          No headers configured. Add headers to send custom HTTP headers with your
          webhook requests.
        </p>
      )}

      {headers.map((header, index) => (
        <div key={index} className='flex items-center gap-2'>
          <Input
            value={header.key}
            onChange={(e) => update(index, 'key', e.target.value)}
            aria-label='Header name'
            className='flex-1'
          />
          <Input
            value={header.value}
            onChange={(e) => update(index, 'value', e.target.value)}
            aria-label='Header value'
            className='flex-1'
          />
          <Button
            variant='ghost'
            size='icon'
            aria-label={`Remove the header ${header.key}`}
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
            placeholder='X-Api-Key'
            aria-label='New header name'
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
            placeholder='value'
            aria-label='New header value'
            className='flex-1 font-mono-ui text-xs'
          />
          <Button size='sm' onClick={commit} disabled={!draftKey || !draftValue}>
            Add
          </Button>
          <Button
            variant='ghost'
            size='icon'
            aria-label='Cancel'
            onClick={cancel}
            className='size-8 text-neutral-400 dark:text-neutral-500'
          >
            <X />
          </Button>
        </div>
      ) : (
        <Button variant='outline' size='sm' onClick={() => setAdding(true)}>
          <Plus /> Add a header
        </Button>
      )}
    </div>
  )
}
