import { useState, type ReactNode } from 'react'
import { Plus, X } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { FieldRow, Section, SwitchField } from '@/components/kit'
import { REALM_NAMESPACE } from '../realm-namespace'
import { isValidCimdHost, isValidResource, validateEntry } from '../mcp-validation'
import type { OwnerClientOption, OwnersByResource } from '../resource-owners'

const NO_OWNER = '__none__'

export interface McpDraft {
  cimdEnabled: boolean
  dcrEnabled: boolean
  cimdAllowedHosts: string[]
  allowedResources: string[]
  owners: OwnersByResource
}

export interface RealmMcpTabProps {
  value: McpDraft
  ownerClients: OwnerClientOption[]
  onChange: (patch: Partial<McpDraft>) => void
}

interface EntryListProps {
  id: string
  entries: string[]
  placeholder: string
  addLabel: string
  removeLabel: (entry: string) => string
  invalidMessage: string
  duplicateMessage: string
  isValid: (value: string) => boolean
  renderExtra?: (entry: string) => ReactNode
  onChange: (next: string[]) => void
}

function EntryList({
  id,
  entries,
  placeholder,
  addLabel,
  removeLabel,
  invalidMessage,
  duplicateMessage,
  isValid,
  renderExtra,
  onChange,
}: EntryListProps) {
  const [draft, setDraft] = useState('')
  const [error, setError] = useState<string | null>(null)

  const add = () => {
    const entry = draft.trim()
    if (!entry) return
    const failure = validateEntry(entry, entries, isValid)
    if (failure) {
      setError(failure === 'duplicate' ? duplicateMessage : invalidMessage)
      return
    }
    onChange([...entries, entry])
    setDraft('')
    setError(null)
  }

  return (
    <div className='flex max-w-xl flex-col gap-2'>
      <div className='flex items-center gap-2'>
        <Input
          id={id}
          value={draft}
          placeholder={placeholder}
          aria-invalid={error !== null}
          onChange={(e) => {
            setDraft(e.target.value)
            setError(null)
          }}
          onKeyDown={(e) => {
            if (e.key !== 'Enter') return
            e.preventDefault()
            add()
          }}
          className='font-mono-ui'
        />
        <Button type='button' variant='outline' onClick={add} disabled={!draft.trim()}>
          <Plus /> {addLabel}
        </Button>
      </div>
      {error && <p className='text-xs text-fk-danger'>{error}</p>}
      {entries.length > 0 && (
        <ul className='flex flex-col gap-1'>
          {entries.map((entry) => (
            <li
              key={entry}
              className='flex items-center justify-between gap-2 rounded-md border border-neutral-200 px-2.5 py-1.5 dark:border-neutral-800'
            >
              <span className='min-w-0 flex-1 truncate font-mono-ui text-xs text-neutral-700 dark:text-neutral-300'>
                {entry}
              </span>
              {renderExtra?.(entry)}
              <button
                type='button'
                aria-label={removeLabel(entry)}
                onClick={() => onChange(entries.filter((item) => item !== entry))}
                className='shrink-0 text-neutral-400 hover:text-neutral-700 dark:hover:text-neutral-200'
              >
                <X className='h-3.5 w-3.5' />
              </button>
            </li>
          ))}
        </ul>
      )}
    </div>
  )
}

export default function RealmMcpTab({ value, ownerClients, onChange }: RealmMcpTabProps) {
  const { t } = useTranslation(REALM_NAMESPACE)

  return (
    <Section title={t('mcp.title')} description={t('mcp.description')}>
      <FieldRow label={t('mcp.cimd.label')} description={t('mcp.cimd.description')}>
        <SwitchField
          checked={value.cimdEnabled}
          onCheckedChange={(v) => onChange({ cimdEnabled: v })}
        />
      </FieldRow>

      <FieldRow label={t('mcp.dcr.label')} description={t('mcp.dcr.description')}>
        <SwitchField
          checked={value.dcrEnabled}
          onCheckedChange={(v) => onChange({ dcrEnabled: v })}
        />
      </FieldRow>

      <FieldRow
        label={t('mcp.hosts.label')}
        description={t('mcp.hosts.description')}
        htmlFor='realm-cimd-hosts'
      >
        <EntryList
          id='realm-cimd-hosts'
          entries={value.cimdAllowedHosts}
          placeholder={t('mcp.hosts.placeholder')}
          addLabel={t('mcp.add')}
          removeLabel={(entry) => t('mcp.remove', { entry })}
          invalidMessage={t('validation.cimd_host_invalid')}
          duplicateMessage={t('validation.entry_duplicate')}
          isValid={isValidCimdHost}
          onChange={(next) => onChange({ cimdAllowedHosts: next })}
        />
      </FieldRow>

      <FieldRow
        label={t('mcp.resources.label')}
        description={t('mcp.resources.description')}
        htmlFor='realm-allowed-resources'
      >
        <EntryList
          id='realm-allowed-resources'
          entries={value.allowedResources}
          placeholder={t('mcp.resources.placeholder')}
          addLabel={t('mcp.add')}
          removeLabel={(entry) => t('mcp.remove', { entry })}
          invalidMessage={t('validation.resource_invalid')}
          duplicateMessage={t('validation.entry_duplicate')}
          isValid={isValidResource}
          renderExtra={(entry) => (
            <Select
              value={value.owners[entry] || NO_OWNER}
              onValueChange={(next) =>
                onChange({
                  owners: { ...value.owners, [entry]: next === NO_OWNER ? '' : next },
                })
              }
            >
              <SelectTrigger
                size='sm'
                className='w-48 shrink-0'
                aria-label={t('mcp.resources.owner.aria', { entry })}
              >
                <SelectValue placeholder={t('mcp.resources.owner.none')} />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value={NO_OWNER}>{t('mcp.resources.owner.none')}</SelectItem>
                {ownerClients.map((client) => (
                  <SelectItem key={client.id} value={client.id}>
                    {client.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          )}
          onChange={(next) => onChange({ allowedResources: next })}
        />
      </FieldRow>
    </Section>
  )
}
