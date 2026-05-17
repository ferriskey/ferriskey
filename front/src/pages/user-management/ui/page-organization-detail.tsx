import { Schemas } from '@/api/api.client'
import { Skeleton } from '@/components/ui/skeleton'
import {
  AlertTriangle,
  ArrowLeft,
  Globe,
  Loader2,
  Power,
  Trash2,
  Users,
} from 'lucide-react'
import { useMemo, useState } from 'react'

import Organization = Schemas.Organization

export interface OrganizationDetailValues {
  name: string
  alias: string
  domain: string
  description: string
  enabled: boolean
}

interface Props {
  organization: Organization | null
  memberCount: number
  isLoading: boolean
  isUpdating: boolean
  isDeleting: boolean
  onBack: () => void
  onSave: (values: OrganizationDetailValues) => void
  onDelete: () => void
}

const initials = (name: string) => {
  const parts = name.trim().split(/\s+/)
  if (parts.length >= 2) return `${parts[0][0]}${parts[1][0]}`.toUpperCase()
  return name.slice(0, 2).toUpperCase()
}

const formatDate = (iso: string) => {
  const d = new Date(iso)
  if (Number.isNaN(d.getTime())) return '—'
  return d.toLocaleString(undefined, { dateStyle: 'medium', timeStyle: 'short' })
}

export default function PageOrganizationDetail(props: Props) {
  const { organization, isLoading } = props
  if (isLoading || !organization) {
    return (
      <div className='flex flex-col gap-6 p-8 md:p-12'>
        <Skeleton className='h-4 w-32' />
        <div className='flex items-center gap-4'>
          <Skeleton className='h-16 w-16 rounded-md' />
          <div className='space-y-2'>
            <Skeleton className='h-6 w-48' />
            <Skeleton className='h-3 w-32' />
          </div>
        </div>
        <Skeleton className='h-64 w-full rounded-md' />
      </div>
    )
  }
  return <OrganizationDetailLoaded key={organization.id} {...props} organization={organization} />
}

interface LoadedProps extends Omit<Props, 'organization'> {
  organization: Organization
}

function OrganizationDetailLoaded({
  organization,
  memberCount,
  isUpdating,
  isDeleting,
  onBack,
  onSave,
  onDelete,
}: LoadedProps) {
  const [name, setName] = useState(organization.name)
  const [alias, setAlias] = useState(organization.alias)
  const [domain, setDomain] = useState(organization.domain ?? '')
  const [description, setDescription] = useState(organization.description ?? '')
  const [enabled, setEnabled] = useState(organization.enabled)
  const [confirmingDelete, setConfirmingDelete] = useState(false)

  const aliasValid = /^[a-z0-9_-]+$/.test(alias)
  const dirty = useMemo(
    () =>
      name !== organization.name ||
      alias !== organization.alias ||
      domain !== (organization.domain ?? '') ||
      description !== (organization.description ?? '') ||
      enabled !== organization.enabled,
    [organization, name, alias, domain, description, enabled],
  )
  const canSave = dirty && aliasValid && name.trim().length > 0 && !isUpdating

  const handleSave = (e: React.FormEvent) => {
    e.preventDefault()
    if (!canSave) return
    onSave({
      name: name.trim(),
      alias: alias.trim(),
      domain: domain.trim(),
      description: description.trim(),
      enabled,
    })
  }

  return (
    <div className='flex flex-col gap-8 p-8 md:p-12 max-w-4xl'>
      {/* Header */}
      <div>
        <button
          type='button'
          onClick={onBack}
          className='inline-flex items-center gap-1.5 text-xs text-muted-foreground hover:text-foreground transition-colors mb-4'
        >
          <ArrowLeft className='h-3.5 w-3.5' />
          Back to organizations
        </button>
        <div className='flex items-center gap-4'>
          <div className='h-16 w-16 rounded-md bg-primary/10 text-primary flex items-center justify-center text-lg font-semibold'>
            {initials(organization.name || organization.alias)}
          </div>
          <div>
            <h1 className='text-2xl font-medium tracking-tight flex items-center gap-3'>
              {organization.name}
              {!enabled && (
                <span className='inline-flex items-center gap-1 rounded-md px-2 py-0.5 text-[11px] font-medium bg-muted text-muted-foreground border border-border uppercase tracking-wide'>
                  Disabled
                </span>
              )}
            </h1>
            <p className='text-sm text-muted-foreground mt-1 font-mono'>@{organization.alias}</p>
          </div>
        </div>
      </div>

      {/* Top stats */}
      <div className='grid grid-cols-1 sm:grid-cols-3 gap-3'>
        <SmallStat icon={Users} tone='emerald' label='Members' value={memberCount} />
        <SmallStat icon={Globe} tone='blue' label='Custom domain' value={organization.domain ? 1 : 0} />
        <SmallStat icon={Power} tone={enabled ? 'emerald' : 'muted'} label='Status' value={enabled ? 'On' : 'Off'} />
      </div>

      {/* Profile */}
      <Section title='Profile' description='Basic information used to identify this organization.'>
        <form onSubmit={handleSave} className='flex flex-col gap-5'>
          <div className='grid grid-cols-1 sm:grid-cols-2 gap-4'>
            <Field label='Name' required>
              <input
                type='text'
                value={name}
                onChange={(e) => setName(e.target.value)}
                placeholder='Acme Inc.'
                className='w-full rounded-md border border-border bg-background px-3 py-2 text-sm outline-none placeholder:text-muted-foreground focus:border-primary/40 focus:ring-1 focus:ring-primary/30'
              />
            </Field>
            <Field
              label='Alias'
              required
              hint='Lowercase identifier used in URLs and APIs.'
              error={!aliasValid && alias.length > 0 ? 'Only lowercase letters, numbers, hyphens and underscores.' : undefined}
            >
              <input
                type='text'
                value={alias}
                onChange={(e) => setAlias(e.target.value)}
                placeholder='acme'
                className='w-full rounded-md border border-border bg-background px-3 py-2 text-sm font-mono outline-none placeholder:text-muted-foreground focus:border-primary/40 focus:ring-1 focus:ring-primary/30'
              />
            </Field>
          </div>

          <Field label='Domain' optional hint='Used to auto-route users with matching email to this organization.'>
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

          <Field label='Description' optional>
            <textarea
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              rows={3}
              placeholder='Internal note about this organization.'
              className='w-full resize-none rounded-md border border-border bg-background px-3 py-2 text-sm outline-none placeholder:text-muted-foreground focus:border-primary/40 focus:ring-1 focus:ring-primary/30'
            />
          </Field>

          <label className='flex items-start gap-3 rounded-md border border-border bg-card/40 p-3 cursor-pointer hover:bg-muted/40 transition-colors'>
            <input
              type='checkbox'
              checked={enabled}
              onChange={(e) => setEnabled(e.target.checked)}
              className='mt-0.5 h-4 w-4 rounded border-border accent-primary'
            />
            <div className='flex-1'>
              <div className='flex items-center gap-2'>
                <Power className='h-4 w-4 text-blue-500' />
                <span className='text-sm font-medium'>Organization enabled</span>
              </div>
              <p className='text-xs text-muted-foreground mt-1'>
                Disabled organizations can&apos;t accept new sign-ins or invitations.
              </p>
            </div>
          </label>

          <div className='flex items-center justify-end gap-2 pt-2 border-t border-border'>
            <button
              type='button'
              onClick={() => {
                setName(organization.name)
                setAlias(organization.alias)
                setDomain(organization.domain ?? '')
                setDescription(organization.description ?? '')
                setEnabled(organization.enabled)
              }}
              disabled={!dirty || isUpdating}
              className='rounded-md border border-border bg-background px-4 py-2 text-sm font-medium hover:bg-muted transition-colors disabled:opacity-40 disabled:cursor-not-allowed'
            >
              Reset
            </button>
            <button
              type='submit'
              disabled={!canSave}
              className='inline-flex items-center gap-2 rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 transition-colors disabled:opacity-50 disabled:cursor-not-allowed'
            >
              {isUpdating && <Loader2 className='h-3.5 w-3.5 animate-spin' />}
              {isUpdating ? 'Saving…' : 'Save changes'}
            </button>
          </div>
        </form>
      </Section>

      {/* Account info */}
      <Section title='Account info' description='Read-only metadata about this organization.'>
        <dl className='grid grid-cols-1 sm:grid-cols-2 gap-x-6 gap-y-3 text-sm'>
          <InfoRow label='Organization ID' value={<code className='font-mono text-xs'>{organization.id}</code>} />
          <InfoRow label='Realm' value={<code className='font-mono text-xs'>{organization.realm_id}</code>} />
          <InfoRow label='Created' value={formatDate(organization.created_at)} />
          <InfoRow label='Last update' value={formatDate(organization.updated_at)} />
        </dl>
      </Section>

      {/* Danger zone */}
      <Section title='Danger zone' description='Destructive actions. These cannot be undone.' tone='destructive'>
        <div className='flex items-center justify-between gap-4 rounded-md border border-red-500/30 bg-red-500/5 p-4'>
          <div className='flex items-start gap-3 min-w-0'>
            <AlertTriangle className='h-4 w-4 text-red-500 mt-0.5 shrink-0' />
            <div>
              <p className='text-sm font-medium'>Delete this organization</p>
              <p className='text-xs text-muted-foreground mt-0.5'>
                Removes the organization and unlinks its members. Identities are preserved.
              </p>
            </div>
          </div>
          {confirmingDelete ? (
            <div className='flex items-center gap-2'>
              <button
                type='button'
                onClick={() => setConfirmingDelete(false)}
                className='rounded-md border border-border bg-background px-3 py-1.5 text-xs font-medium hover:bg-muted transition-colors'
              >
                Cancel
              </button>
              <button
                type='button'
                onClick={onDelete}
                disabled={isDeleting}
                className='inline-flex items-center gap-1.5 rounded-md bg-red-500 px-3 py-1.5 text-xs font-medium text-white hover:bg-red-600 transition-colors disabled:opacity-50'
              >
                {isDeleting && <Loader2 className='h-3 w-3 animate-spin' />}
                {isDeleting ? 'Deleting…' : 'Confirm delete'}
              </button>
            </div>
          ) : (
            <button
              type='button'
              onClick={() => setConfirmingDelete(true)}
              className='inline-flex items-center gap-1.5 rounded-md border border-red-500/40 bg-background px-3 py-1.5 text-xs font-medium text-red-600 hover:bg-red-500/10 transition-colors'
            >
              <Trash2 className='h-3 w-3' />
              Delete organization
            </button>
          )}
        </div>
      </Section>
    </div>
  )
}

interface SectionProps {
  title: string
  description?: string
  tone?: 'default' | 'destructive'
  children: React.ReactNode
}

function Section({ title, description, tone = 'default', children }: SectionProps) {
  return (
    <section className='flex flex-col gap-4'>
      <div>
        <h2
          className={`text-sm font-semibold ${tone === 'destructive' ? 'text-red-600' : 'text-foreground'}`}
        >
          {title}
        </h2>
        {description && <p className='text-xs text-muted-foreground mt-1'>{description}</p>}
      </div>
      {children}
    </section>
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

function InfoRow({ label, value }: { label: string; value: React.ReactNode }) {
  return (
    <div className='flex flex-col gap-0.5'>
      <dt className='text-xs text-muted-foreground'>{label}</dt>
      <dd className='text-sm'>{value}</dd>
    </div>
  )
}

interface SmallStatProps {
  icon: React.ComponentType<{ className?: string }>
  tone: 'emerald' | 'blue' | 'amber' | 'muted'
  label: string
  value: number | string
}

function SmallStat({ icon: Icon, tone, label, value }: SmallStatProps) {
  const tones: Record<SmallStatProps['tone'], { bg: string; fg: string }> = {
    emerald: { bg: 'bg-emerald-500/10', fg: 'text-emerald-500' },
    blue: { bg: 'bg-blue-500/10', fg: 'text-blue-500' },
    amber: { bg: 'bg-amber-500/10', fg: 'text-amber-500' },
    muted: { bg: 'bg-muted', fg: 'text-muted-foreground' },
  }
  const t = tones[tone]
  return (
    <div className='flex items-center gap-3 rounded-md border border-border bg-card/40 px-4 py-3'>
      <div className={`h-9 w-9 rounded-md flex items-center justify-center ${t.bg}`}>
        <Icon className={`h-4 w-4 ${t.fg}`} />
      </div>
      <div className='min-w-0'>
        <div className='text-xl font-semibold tabular-nums leading-none'>{value}</div>
        <p className='text-xs text-muted-foreground mt-1 truncate'>{label}</p>
      </div>
    </div>
  )
}
