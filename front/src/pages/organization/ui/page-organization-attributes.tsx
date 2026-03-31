import { Schemas } from '@/api/api.client.ts'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Skeleton } from '@/components/ui/skeleton'
import { Pencil, Plus, Trash2, X, Check } from 'lucide-react'
import { useState } from 'react'
import OrganizationAttribute = Schemas.OrganizationAttribute

export interface PageOrganizationAttributesProps {
  attributes: OrganizationAttribute[]
  isLoading: boolean
  onUpsert: (key: string, value: string) => void
  onDelete: (key: string) => void
}

interface AttributeRowProps {
  attribute: OrganizationAttribute
  onEdit: (key: string, value: string) => void
  onDelete: (key: string) => void
}

function AttributeRow({ attribute, onEdit, onDelete }: AttributeRowProps) {
  const [editing, setEditing] = useState(false)
  const [draft, setDraft] = useState(attribute.value)

  const handleSave = () => {
    if (draft.trim()) {
      onEdit(attribute.key, draft.trim())
    }
    setEditing(false)
  }

  const handleCancel = () => {
    setDraft(attribute.value)
    setEditing(false)
  }

  return (
    <div className='flex items-center justify-between px-8 py-3 border-b last:border-0'>
      <div className='flex items-center gap-6 flex-1 min-w-0'>
        <span className='font-mono text-sm text-foreground w-48 shrink-0 truncate'>
          {attribute.key}
        </span>
        {editing ? (
          <Input
            autoFocus
            value={draft}
            onChange={(e) => setDraft(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === 'Enter') handleSave()
              if (e.key === 'Escape') handleCancel()
            }}
            className='h-8 text-sm max-w-xs'
          />
        ) : (
          <span className='text-sm text-muted-foreground truncate'>{attribute.value}</span>
        )}
      </div>
      <div className='flex items-center gap-1 shrink-0 ml-4'>
        {editing ? (
          <>
            <Button variant='ghost' size='icon' className='h-8 w-8' onClick={handleSave}>
              <Check className='h-4 w-4 text-green-600' />
            </Button>
            <Button variant='ghost' size='icon' className='h-8 w-8' onClick={handleCancel}>
              <X className='h-4 w-4' />
            </Button>
          </>
        ) : (
          <>
            <Button
              variant='ghost'
              size='icon'
              className='h-8 w-8'
              onClick={() => setEditing(true)}
            >
              <Pencil className='h-4 w-4' />
            </Button>
            <Button
              variant='ghost'
              size='icon'
              className='h-8 w-8'
              onClick={() => onDelete(attribute.key)}
            >
              <Trash2 className='h-4 w-4 text-destructive' />
            </Button>
          </>
        )}
      </div>
    </div>
  )
}

interface AddAttributeRowProps {
  onAdd: (key: string, value: string) => void
  onCancel: () => void
}

function AddAttributeRow({ onAdd, onCancel }: AddAttributeRowProps) {
  const [key, setKey] = useState('')
  const [value, setValue] = useState('')

  const canSave = key.trim().length > 0 && value.trim().length > 0

  const handleSave = () => {
    if (canSave) {
      onAdd(key.trim(), value.trim())
    }
  }

  return (
    <div className='flex items-center gap-4 px-8 py-3 border-b bg-muted/30'>
      <Input
        autoFocus
        placeholder='Key'
        value={key}
        onChange={(e) => setKey(e.target.value)}
        onKeyDown={(e) => e.key === 'Escape' && onCancel()}
        className='h-8 text-sm font-mono w-48 shrink-0'
      />
      <Input
        placeholder='Value'
        value={value}
        onChange={(e) => setValue(e.target.value)}
        onKeyDown={(e) => {
          if (e.key === 'Enter') handleSave()
          if (e.key === 'Escape') onCancel()
        }}
        className='h-8 text-sm max-w-xs'
      />
      <div className='flex items-center gap-1 ml-auto'>
        <Button
          variant='ghost'
          size='icon'
          className='h-8 w-8'
          onClick={handleSave}
          disabled={!canSave}
        >
          <Check className='h-4 w-4 text-green-600' />
        </Button>
        <Button variant='ghost' size='icon' className='h-8 w-8' onClick={onCancel}>
          <X className='h-4 w-4' />
        </Button>
      </div>
    </div>
  )
}

export default function PageOrganizationAttributes({
  attributes,
  isLoading,
  onUpsert,
  onDelete,
}: PageOrganizationAttributesProps) {
  const [adding, setAdding] = useState(false)

  return (
    <div>
      <div className='flex items-center justify-between mb-3'>
        <div>
          <p className='text-xs text-muted-foreground mb-0.5'>Custom key-value metadata</p>
          <h2 className='text-base font-semibold'>Attributes ({attributes.length})</h2>
        </div>
        {!adding && (
          <Button size='sm' onClick={() => setAdding(true)}>
            <Plus className='h-4 w-4' />
            Add attribute
          </Button>
        )}
      </div>

      <div className='-mx-8 border-t border-b overflow-hidden'>
        {/* Column headers */}
        <div className='flex items-center px-8 py-2 bg-muted/40 border-b'>
          <span className='text-xs font-medium text-muted-foreground uppercase tracking-wide w-48 shrink-0'>
            Key
          </span>
          <span className='text-xs font-medium text-muted-foreground uppercase tracking-wide'>
            Value
          </span>
        </div>

        {adding && (
          <AddAttributeRow
            onAdd={(k, v) => {
              onUpsert(k, v)
              setAdding(false)
            }}
            onCancel={() => setAdding(false)}
          />
        )}

        {isLoading ? (
          Array.from({ length: 4 }).map((_, i) => (
            <div key={i} className='flex items-center justify-between px-8 py-3 border-b'>
              <div className='flex items-center gap-6'>
                <Skeleton className='h-4 w-36' />
                <Skeleton className='h-4 w-48' />
              </div>
              <div className='flex gap-1'>
                <Skeleton className='h-8 w-8 rounded-md' />
                <Skeleton className='h-8 w-8 rounded-md' />
              </div>
            </div>
          ))
        ) : attributes.length === 0 && !adding ? (
          <div className='flex items-center justify-center h-20 text-sm text-muted-foreground'>
            No attributes defined for this organization.
          </div>
        ) : (
          attributes.map((attr) => (
            <AttributeRow
              key={attr.id}
              attribute={attr}
              onEdit={onUpsert}
              onDelete={onDelete}
            />
          ))
        )}
      </div>
    </div>
  )
}
