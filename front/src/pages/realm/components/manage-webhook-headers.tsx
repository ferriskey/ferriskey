import { Button } from '@/components/ui/button'
import { InputText } from '@/components/ui/input-text'
import { Trash2, PlusIcon, Check, X } from 'lucide-react'
import { useState } from 'react'

export interface WebhookHeader {
  key: string
  value: string
}

export interface ManageWebhookHeadersProps {
  headers: WebhookHeader[]
  onChange: (headers: WebhookHeader[]) => void
}

export default function ManageWebhookHeaders({ headers, onChange }: ManageWebhookHeadersProps) {
  const [newHeaderKey, setNewHeaderKey] = useState<string>('')
  const [newHeaderValue, setNewHeaderValue] = useState<string>('')
  const [isAddingHeader, setIsAddingHeader] = useState<boolean>(false)

  const handleDeleteHeader = (index: number) => {
    const updatedHeaders = headers.filter((_, i) => i !== index)
    onChange(updatedHeaders)
  }

  const handleAddHeader = () => {
    if (!newHeaderKey || !newHeaderValue) return

    const updatedHeaders = [...headers, { key: newHeaderKey, value: newHeaderValue }]
    onChange(updatedHeaders)

    setNewHeaderKey('')
    setNewHeaderValue('')
    setIsAddingHeader(false)
  }

  const handleCancelAdd = () => {
    setNewHeaderKey('')
    setNewHeaderValue('')
    setIsAddingHeader(false)
  }

  const handleUpdateHeader = (index: number, field: 'key' | 'value', value: string) => {
    const updatedHeaders = headers.map((header, i) => {
      if (i === index) {
        return { ...header, [field]: value }
      }
      return header
    })
    onChange(updatedHeaders)
  }

  return (
    <div className='flex flex-col gap-4'>
      {headers.length === 0 && !isAddingHeader && (
        <p className='text-sm text-muted-foreground'>
          No headers configured. Add headers to send custom HTTP headers with your webhook requests.
        </p>
      )}

      {headers.map((header, index) => (
        <div key={index} className='flex gap-2 items-end'>
          <div className='flex-1'>
            <InputText
              name={`header_key_${index}`}
              label='Header Key'
              value={header.key}
              onChange={(value) => handleUpdateHeader(index, 'key', value as string)}
            />
          </div>

          <div className='flex-1'>
            <InputText
              name={`header_value_${index}`}
              label='Header Value'
              value={header.value}
              onChange={(value) => handleUpdateHeader(index, 'value', value as string)}
            />
          </div>

          <div className='pb-1'>
            <Button
              className='text-red-500'
              variant='ghost'
              size='icon'
              onClick={() => handleDeleteHeader(index)}
            >
              <Trash2 size={14} />
            </Button>
          </div>
        </div>
      ))}

      {isAddingHeader && (
        <div className='flex flex-col gap-3'>
          <div className='flex gap-2 items-end'>
            <div className='flex-1'>
              <InputText
                name='new_header_key'
                label='Header Key'
                onChange={(value) => setNewHeaderKey(value as string)}
                value={newHeaderKey}
              />
            </div>

            <div className='flex-1'>
              <InputText
                name='new_header_value'
                label='Header Value'
                onChange={(value) => setNewHeaderValue(value as string)}
                value={newHeaderValue}
              />
            </div>
          </div>

          <div className='flex gap-2'>
            <Button
              onClick={handleAddHeader}
              disabled={!newHeaderKey || !newHeaderValue}
              variant='outline'
              size='sm'
              className='w-fit'
            >
              <Check className='w-4 h-4 mr-2' />
              Save
            </Button>
            <Button
              onClick={handleCancelAdd}
              variant='ghost'
              size='sm'
              className='w-fit'
            >
              <X className='w-4 h-4 mr-2' />
              Cancel
            </Button>
          </div>
        </div>
      )}

      {!isAddingHeader && (
        <Button
          onClick={() => setIsAddingHeader(true)}
          variant='outline'
          className='w-fit'
        >
          <PlusIcon className='w-4 h-4 mr-2' />
          Add Header
        </Button>
      )}
    </div>
  )
}
