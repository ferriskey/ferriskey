import { Schemas } from '@/api/api.client'
import { Permissions } from '@/api/core.interface'
import { Skeleton } from '@/components/ui/skeleton'
import {
  AlertTriangle,
  ArrowLeft,
  Check,
  Loader2,
  Lock,
  ShieldCheck,
  ShieldUser,
  Trash2,
} from 'lucide-react'
import { useState } from 'react'
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import { z } from 'zod'

import Role = Schemas.Role

const roleDetailSchema = z.object({
  name: z.string().trim().min(1, 'Name is required'),
  description: z.string().trim().optional().default(''),
  permissions: z.array(z.string()).default([]),
})

type RoleDetailSchema = z.infer<typeof roleDetailSchema>

export interface RoleDetailValues {
  name: string
  description: string
  permissions: string[]
}

interface Props {
  role: Role | null
  isLoading: boolean
  isUpdating: boolean
  isDeleting: boolean
  onBack: () => void
  onSave: (values: RoleDetailValues) => void
  onDelete: () => void
}

const PERMISSION_GROUPS: { label: string; permissions: Permissions[] }[] = [
  {
    label: 'Identities',
    permissions: [Permissions.ViewUsers, Permissions.QueryUsers, Permissions.ManageUsers],
  },
  {
    label: 'Clients',
    permissions: [
      Permissions.ViewClients,
      Permissions.QueryClients,
      Permissions.ManageClients,
      Permissions.CreateClient,
    ],
  },
  {
    label: 'Roles & permissions',
    permissions: [Permissions.ViewRoles, Permissions.ManageRoles],
  },
  {
    label: 'Realm',
    permissions: [Permissions.ViewRealm, Permissions.QueryRealms, Permissions.ManageRealm],
  },
  {
    label: 'Identity providers',
    permissions: [Permissions.ViewIdentityProviders, Permissions.ManageIdentityProviders],
  },
  {
    label: 'Authorization',
    permissions: [Permissions.ViewAuthorization, Permissions.ManageAuthorization],
  },
  {
    label: 'Audit & events',
    permissions: [Permissions.ViewEvents, Permissions.ManageEvents],
  },
  {
    label: 'Groups',
    permissions: [Permissions.QueryGroups],
  },
]

const formatDate = (iso: string) => {
  const d = new Date(iso)
  if (Number.isNaN(d.getTime())) return '—'
  return d.toLocaleString(undefined, { dateStyle: 'medium', timeStyle: 'short' })
}

export default function PageRoleDetail(props: Props) {
  const { role, isLoading } = props
  if (isLoading || !role) {
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
  return <RoleDetailLoaded key={role.id} {...props} role={role} />
}

interface LoadedProps extends Omit<Props, 'role'> {
  role: Role
}

function RoleDetailLoaded({ role, isUpdating, isDeleting, onBack, onSave, onDelete }: LoadedProps) {
  const defaultValues = {
    name: role.name,
    description: role.description ?? '',
    permissions: role.permissions ?? [],
  }

  const {
    register,
    handleSubmit,
    watch,
    setValue,
    reset,
    formState: { isDirty, isValid },
  } = useForm<RoleDetailSchema>({
    resolver: zodResolver(roleDetailSchema),
    defaultValues,
    mode: 'onChange',
  })

  const permissions = watch('permissions')
  const [confirmingDelete, setConfirmingDelete] = useState(false)

  const isClientScope = !!role.client_id
  const canSave = isDirty && isValid && !isUpdating

  const togglePerm = (perm: string) => {
    const next = permissions.includes(perm)
      ? permissions.filter((p) => p !== perm)
      : [...permissions, perm]
    setValue('permissions', next, { shouldValidate: true, shouldDirty: true })
  }

  const handleSave = handleSubmit((values) => {
    onSave({
      name: values.name.trim(),
      description: (values.description ?? '').trim(),
      permissions: values.permissions,
    })
  })

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
          Back to roles
        </button>
        <div className='flex items-center gap-4'>
          <div
            className={`h-16 w-16 rounded-md flex items-center justify-center ${
              isClientScope ? 'bg-violet-500/10 text-violet-500' : 'bg-primary/10 text-primary'
            }`}
          >
            <ShieldUser className='h-7 w-7' />
          </div>
          <div>
            <h1 className='text-2xl font-medium tracking-tight flex items-center gap-3'>
              {role.name}
              {isClientScope ? (
                <span className='inline-flex items-center gap-1 rounded-md px-2 py-0.5 text-[11px] font-medium bg-violet-500/10 text-violet-500 border border-violet-500/30 uppercase tracking-wide'>
                  <Lock className='h-3 w-3' />
                  Client
                </span>
              ) : (
                <span className='inline-flex items-center gap-1 rounded-md px-2 py-0.5 text-[11px] font-medium bg-emerald-500/10 text-emerald-600 border border-emerald-500/30 uppercase tracking-wide'>
                  <ShieldCheck className='h-3 w-3' />
                  Realm
                </span>
              )}
            </h1>
            <p className='text-sm text-muted-foreground mt-1'>
              {(role.permissions?.length ?? 0)} permission{(role.permissions?.length ?? 0) === 1 ? '' : 's'} granted
            </p>
          </div>
        </div>
      </div>

      <form onSubmit={handleSave} className='flex flex-col gap-8'>
        {/* Profile */}
        <Section title='Profile' description='How this role is identified when assigning it.'>
          <div className='flex flex-col gap-5'>
            <Field label='Name' required>
              <input
                type='text'
                {...register('name')}
                placeholder='Customer Support'
                className='w-full rounded-md border border-border bg-background px-3 py-2 text-sm outline-none placeholder:text-muted-foreground focus:border-primary/40 focus:ring-1 focus:ring-primary/30'
              />
            </Field>
            <Field label='Description' optional>
              <textarea
                {...register('description')}
                rows={2}
                placeholder='Can read customer data and reset passwords.'
                className='w-full resize-none rounded-md border border-border bg-background px-3 py-2 text-sm outline-none placeholder:text-muted-foreground focus:border-primary/40 focus:ring-1 focus:ring-primary/30'
              />
            </Field>
          </div>
        </Section>

        {/* Permissions */}
        <Section
          title='Permissions'
          description={`${permissions.length} permission${permissions.length === 1 ? '' : 's'} selected. Toggle each chip to grant or revoke.`}
        >
          <div className='flex flex-col gap-4'>
            {PERMISSION_GROUPS.map((group) => (
              <div key={group.label} className='flex flex-col gap-2'>
                <p className='text-[11px] font-semibold uppercase tracking-wider text-muted-foreground'>
                  {group.label}
                </p>
                <div className='flex flex-wrap gap-2'>
                  {group.permissions.map((perm) => {
                    const checked = permissions.includes(perm)
                    return (
                      <button
                        type='button'
                        key={perm}
                        onClick={() => togglePerm(perm)}
                        className={`inline-flex items-center gap-1.5 rounded-md border px-2.5 py-1 text-xs font-mono transition-colors ${
                          checked
                            ? 'bg-primary/10 text-primary border-primary/40'
                            : 'bg-background text-muted-foreground border-border hover:bg-muted'
                        }`}
                      >
                        {checked && <Check className='h-3 w-3' />}
                        {perm}
                      </button>
                    )
                  })}
                </div>
              </div>
            ))}
          </div>
        </Section>

        {/* Save bar */}
        <div className='flex items-center justify-between gap-3 pt-4 border-t border-border'>
          <p className='text-xs text-muted-foreground'>
            Will grant{' '}
            <span className='font-semibold text-foreground tabular-nums'>{permissions.length}</span>{' '}
            permission{permissions.length === 1 ? '' : 's'}.
          </p>
          <div className='flex items-center gap-2'>
            <button
              type='button'
              onClick={() => reset(defaultValues)}
              disabled={!isDirty || isUpdating}
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
        </div>
      </form>

      {/* Account info */}
      <Section title='Account info' description='Read-only metadata about this role.'>
        <dl className='grid grid-cols-1 sm:grid-cols-2 gap-x-6 gap-y-3 text-sm'>
          <InfoRow label='Role ID' value={<code className='font-mono text-xs'>{role.id}</code>} />
          <InfoRow label='Realm' value={<code className='font-mono text-xs'>{role.realm_id}</code>} />
          {isClientScope && (
            <InfoRow label='Client' value={<code className='font-mono text-xs'>{role.client_id}</code>} />
          )}
          <InfoRow label='Created' value={formatDate(role.created_at)} />
          <InfoRow label='Last update' value={formatDate(role.updated_at)} />
        </dl>
      </Section>

      {/* Danger zone */}
      <Section title='Danger zone' description='Destructive actions. These cannot be undone.' tone='destructive'>
        <div className='flex items-center justify-between gap-4 rounded-md border border-red-500/30 bg-red-500/5 p-4'>
          <div className='flex items-start gap-3 min-w-0'>
            <AlertTriangle className='h-4 w-4 text-red-500 mt-0.5 shrink-0' />
            <div>
              <p className='text-sm font-medium'>Delete this role</p>
              <p className='text-xs text-muted-foreground mt-0.5'>
                Identities currently assigned to this role will lose its permissions immediately.
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
              Delete role
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
        <h2 className={`text-sm font-semibold ${tone === 'destructive' ? 'text-red-600' : 'text-foreground'}`}>
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
  children: React.ReactNode
}

function Field({ label, hint, required, optional, children }: FieldProps) {
  return (
    <label className='flex flex-col gap-1.5'>
      <div className='flex items-center gap-2'>
        <span className='text-sm font-medium'>{label}</span>
        {required && <span className='text-[10px] uppercase tracking-wider text-muted-foreground'>required</span>}
        {optional && <span className='text-[10px] uppercase tracking-wider text-muted-foreground'>optional</span>}
      </div>
      {children}
      {hint && <p className='text-xs text-muted-foreground'>{hint}</p>}
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
