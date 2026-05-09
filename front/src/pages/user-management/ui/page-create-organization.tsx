import { ArrowLeft, Building2, Globe } from 'lucide-react'
import { useMemo, useState } from 'react'

interface Props {
  onCancel: () => void
  onSubmit: (values: { name: string; alias: string; domain: string; description: string }) => void
  isSubmitting: boolean
}

const slugify = (s: string) =>
  s
    .toLowerCase()
    .normalize('NFD')
    .replace(/[\u0300-\u036f]/g, '')
    .replace(/[^a-z0-9-_]+/g, '-')
    .replace(/^-+|-+$/g, '')

export default function PageCreateOrganization({ onCancel, onSubmit, isSubmitting }: Props) {
  const [name, setName] = useState('')
  const [aliasOverride, setAliasOverride] = useState<string | null>(null)
  const [domain, setDomain] = useState('')
  const [description, setDescription] = useState('')

  const alias = aliasOverride ?? slugify(name)
  const aliasValid = useMemo(() => /^[a-z0-9_-]+$/.test(alias), [alias])
  const canSubmit = name.trim().length > 0 && alias.trim().length > 0 && aliasValid && !isSubmitting

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    if (!canSubmit) return
    onSubmit({ name: name.trim(), alias: alias.trim(), domain: domain.trim(), description: description.trim() })
  }

  return (
    <form onSubmit={handleSubmit} className='flex flex-col gap-8 p-8 md:p-12 max-w-2xl'>
      {/* Header */}
      <div>
        <button
          type='button'
          onClick={onCancel}
          className='inline-flex items-center gap-1.5 text-xs text-muted-foreground hover:text-foreground transition-colors mb-4'
        >
          <ArrowLeft className='h-3.5 w-3.5' />
          Back to organizations
        </button>
        <div className='flex items-start gap-4'>
          <div className='h-12 w-12 rounded-md bg-primary/10 flex items-center justify-center'>
            <Building2 className='h-5 w-5 text-primary' />
          </div>
          <div>
            <h1 className='text-2xl font-medium tracking-tight'>Create organization</h1>
            <p className='text-sm text-muted-foreground mt-1'>
              Group identities into a B2B tenant. You can attach a domain and customize sign-in flows later.
            </p>
          </div>
        </div>
      </div>

      {/* Fields */}
      <div className='flex flex-col gap-5'>
        <Field
          label='Name'
          required
          hint='How users see this organization (e.g. "Acme Inc.").'
        >
          <input
            type='text'
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder='Acme Inc.'
            autoFocus
            className='w-full rounded-md border border-border bg-background px-3 py-2 text-sm outline-none placeholder:text-muted-foreground focus:border-primary/40 focus:ring-1 focus:ring-primary/30'
          />
        </Field>

        <Field
          label='Alias'
          required
          hint='Lowercase identifier used in URLs and APIs. Auto-derived from the name.'
          error={!aliasValid && alias.length > 0 ? 'Only lowercase letters, numbers, hyphens and underscores.' : undefined}
        >
          <input
            type='text'
            value={alias}
            onChange={(e) => setAliasOverride(e.target.value)}
            placeholder='acme'
            className='w-full rounded-md border border-border bg-background px-3 py-2 text-sm font-mono outline-none placeholder:text-muted-foreground focus:border-primary/40 focus:ring-1 focus:ring-primary/30'
          />
        </Field>

        <Field
          label='Domain'
          optional
          hint='Optional. Used to auto-route users with matching email to this organization.'
        >
          <div className='relative'>
            <Globe className='pointer-events-none absolute left-3 top-1/2 -translate-y-1/2 h-3.5 w-3.5 text-muted-foreground' />
            <input
              type='text'
              value={domain}
              onChange={(e) => setDomain(e.target.value)}
              placeholder='acme.com'
              className='w-full rounded-md border border-border bg-background pl-9 pr-3 py-2 text-sm outline-none placeholder:text-muted-foreground focus:border-primary/40 focus:ring-1 focus:ring-primary/30'
            />
          </div>
        </Field>

        <Field label='Description' optional hint='Internal note to remember what this organization is about.'>
          <textarea
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            rows={3}
            placeholder='Customer in the SaaS plan, kicked-off Q1 2026.'
            className='w-full resize-none rounded-md border border-border bg-background px-3 py-2 text-sm outline-none placeholder:text-muted-foreground focus:border-primary/40 focus:ring-1 focus:ring-primary/30'
          />
        </Field>
      </div>

      {/* Actions */}
      <div className='flex items-center justify-end gap-2 pt-2 border-t border-border'>
        <button
          type='button'
          onClick={onCancel}
          className='rounded-md border border-border bg-background px-4 py-2 text-sm font-medium hover:bg-muted transition-colors'
        >
          Cancel
        </button>
        <button
          type='submit'
          disabled={!canSubmit}
          className='rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 transition-colors disabled:opacity-50 disabled:cursor-not-allowed'
        >
          {isSubmitting ? 'Creating…' : 'Create organization'}
        </button>
      </div>
    </form>
  )
}

interface FieldProps {
  label: string
  hint?: string
  required?: boolean
  optional?: boolean
  error?: string
  children: React.ReactNode
}

function Field({ label, hint, required, optional, error, children }: FieldProps) {
  return (
    <label className='flex flex-col gap-1.5'>
      <div className='flex items-center gap-2'>
        <span className='text-sm font-medium'>{label}</span>
        {required && <span className='text-[10px] uppercase tracking-wider text-muted-foreground'>required</span>}
        {optional && <span className='text-[10px] uppercase tracking-wider text-muted-foreground'>optional</span>}
      </div>
      {children}
      {error ? (
        <p className='text-xs text-red-500'>{error}</p>
      ) : (
        hint && <p className='text-xs text-muted-foreground'>{hint}</p>
      )}
    </label>
  )
}
