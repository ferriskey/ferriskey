import { Permissions } from '@/api/core.interface'
import { ArrowLeft, Check, Eye, Pencil, Settings2, ShieldUser } from 'lucide-react'
import { useMemo } from 'react'
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import { z } from 'zod'

export interface CreateRoleValues {
  name: string
  description: string
  permissions: string[]
}

interface Props {
  onCancel: () => void
  onSubmit: (values: CreateRoleValues) => void
  isSubmitting: boolean
}

type PresetKey = 'viewer' | 'editor' | 'admin' | 'custom'

interface Preset {
  key: PresetKey
  label: string
  description: string
  icon: React.ComponentType<{ className?: string }>
  tone: 'blue' | 'amber' | 'emerald' | 'muted'
  permissions: Permissions[]
}

const VIEWER_PERMS: Permissions[] = [
  Permissions.ViewUsers,
  Permissions.ViewClients,
  Permissions.ViewRoles,
  Permissions.ViewRealm,
  Permissions.ViewEvents,
  Permissions.ViewIdentityProviders,
  Permissions.ViewAuthorization,
  Permissions.QueryUsers,
  Permissions.QueryClients,
  Permissions.QueryRealms,
  Permissions.QueryGroups,
]

const EDITOR_PERMS: Permissions[] = [
  ...VIEWER_PERMS,
  Permissions.ManageUsers,
  Permissions.ManageRoles,
  Permissions.ManageClients,
  Permissions.CreateClient,
]

const ADMIN_PERMS: Permissions[] = Object.values(Permissions)

const presets: Preset[] = [
  {
    key: 'viewer',
    label: 'Viewer',
    description: 'Read-only access to identities, clients, roles and audit data.',
    icon: Eye,
    tone: 'blue',
    permissions: VIEWER_PERMS,
  },
  {
    key: 'editor',
    label: 'Editor',
    description: 'Manage identities, roles and clients. No realm-level changes.',
    icon: Pencil,
    tone: 'amber',
    permissions: EDITOR_PERMS,
  },
  {
    key: 'admin',
    label: 'Admin',
    description: 'Full access including realm and identity provider configuration.',
    icon: ShieldUser,
    tone: 'emerald',
    permissions: ADMIN_PERMS,
  },
  {
    key: 'custom',
    label: 'Custom',
    description: 'Pick exactly which permissions this role grants.',
    icon: Settings2,
    tone: 'muted',
    permissions: [],
  },
]

const toneClasses: Record<Preset['tone'], { bg: string; fg: string; ring: string }> = {
  blue: { bg: 'bg-blue-500/10', fg: 'text-blue-500', ring: 'ring-blue-500/40 border-blue-500/40' },
  amber: { bg: 'bg-amber-500/10', fg: 'text-amber-500', ring: 'ring-amber-500/40 border-amber-500/40' },
  emerald: { bg: 'bg-emerald-500/10', fg: 'text-emerald-500', ring: 'ring-emerald-500/40 border-emerald-500/40' },
  muted: { bg: 'bg-muted', fg: 'text-muted-foreground', ring: 'ring-primary/40 border-primary/40' },
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

const createRoleSchema = z
  .object({
    name: z.string().trim().min(1, 'Name is required'),
    description: z.string().trim().optional().default(''),
    presetKey: z.enum(['viewer', 'editor', 'admin', 'custom']),
    customPerms: z.array(z.nativeEnum(Permissions)).default([]),
  })
  .superRefine((values, ctx) => {
    if (values.presetKey === 'custom' && values.customPerms.length === 0) {
      ctx.addIssue({
        code: z.ZodIssueCode.custom,
        path: ['customPerms'],
        message: 'Pick at least one permission.',
      })
    }
  })

type CreateRoleSchema = z.infer<typeof createRoleSchema>

export default function PageCreateRole({ onCancel, onSubmit, isSubmitting }: Props) {
  const {
    register,
    handleSubmit,
    watch,
    setValue,
    formState: { isValid },
  } = useForm<CreateRoleSchema>({
    resolver: zodResolver(createRoleSchema),
    defaultValues: { name: '', description: '', presetKey: 'viewer', customPerms: [] },
    mode: 'onChange',
  })

  const presetKey = watch('presetKey')
  const customPerms = watch('customPerms')

  const effectivePerms = useMemo<Permissions[]>(() => {
    if (presetKey === 'custom') return customPerms
    return presets.find((p) => p.key === presetKey)?.permissions ?? []
  }, [presetKey, customPerms])

  const togglePerm = (perm: Permissions) => {
    const next = customPerms.includes(perm)
      ? customPerms.filter((p) => p !== perm)
      : [...customPerms, perm]
    setValue('customPerms', next, { shouldValidate: true, shouldDirty: true })
  }

  const submit = handleSubmit((values) => {
    const permissions =
      values.presetKey === 'custom'
        ? values.customPerms
        : (presets.find((p) => p.key === values.presetKey)?.permissions ?? [])

    onSubmit({
      name: values.name.trim(),
      description: (values.description ?? '').trim(),
      permissions: permissions.map((p) => p.toString()),
    })
  })

  const submitDisabled = !isValid || isSubmitting

  return (
    <form onSubmit={submit} className='flex flex-col gap-8 p-8 md:p-12 max-w-3xl'>
      {/* Header */}
      <div>
        <button
          type='button'
          onClick={onCancel}
          className='inline-flex items-center gap-1.5 text-xs text-muted-foreground hover:text-foreground transition-colors mb-4'
        >
          <ArrowLeft className='h-3.5 w-3.5' />
          Back to roles
        </button>
        <div className='flex items-start gap-4'>
          <div className='h-12 w-12 rounded-md bg-primary/10 flex items-center justify-center'>
            <ShieldUser className='h-5 w-5 text-primary' />
          </div>
          <div>
            <h1 className='text-2xl font-medium tracking-tight'>Create role</h1>
            <p className='text-sm text-muted-foreground mt-1'>
              Pick a permission template, or build your own. You can change it later from the role settings.
            </p>
          </div>
        </div>
      </div>

      {/* Identity */}
      <div className='flex flex-col gap-5'>
        <Field label='Name' required hint='How this role appears when assigning it to identities.'>
          <input
            type='text'
            {...register('name')}
            placeholder='Customer Support'
            autoFocus
            className='w-full rounded-md border border-border bg-background px-3 py-2 text-sm outline-none placeholder:text-muted-foreground focus:border-primary/40 focus:ring-1 focus:ring-primary/30'
          />
        </Field>
        <Field label='Description' optional hint='Short summary of what this role can do.'>
          <textarea
            {...register('description')}
            rows={2}
            placeholder='Can read customer data and reset passwords.'
            className='w-full resize-none rounded-md border border-border bg-background px-3 py-2 text-sm outline-none placeholder:text-muted-foreground focus:border-primary/40 focus:ring-1 focus:ring-primary/30'
          />
        </Field>
      </div>

      {/* Presets */}
      <div className='flex flex-col gap-3'>
        <div>
          <h2 className='text-sm font-semibold'>Permission template</h2>
          <p className='text-xs text-muted-foreground mt-0.5'>
            Pick a starting point. Use Custom to fine-tune.
          </p>
        </div>
        <div className='grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-3'>
          {presets.map((p) => {
            const t = toneClasses[p.tone]
            const active = presetKey === p.key
            return (
              <button
                key={p.key}
                type='button'
                onClick={() => setValue('presetKey', p.key, { shouldValidate: true, shouldDirty: true })}
                className={`relative flex flex-col items-start gap-3 rounded-md border bg-card/40 p-4 text-left transition ${
                  active ? 'border-primary ring-1 ring-primary/40 bg-primary/[0.04]' : 'border-border hover:border-primary/30 hover:bg-muted/40'
                }`}
              >
                <div className={`h-9 w-9 rounded-md flex items-center justify-center ${t.bg}`}>
                  <p.icon className={`h-4 w-4 ${t.fg}`} />
                </div>
                <div>
                  <p className='text-sm font-medium'>{p.label}</p>
                  <p className='text-xs text-muted-foreground mt-1'>{p.description}</p>
                </div>
                {p.key !== 'custom' && (
                  <span className='text-[10px] uppercase tracking-wider text-muted-foreground mt-auto'>
                    {p.permissions.length} permission{p.permissions.length > 1 ? 's' : ''}
                  </span>
                )}
                {active && (
                  <span className='absolute top-2 right-2 inline-flex h-5 w-5 items-center justify-center rounded-full bg-primary text-primary-foreground'>
                    <Check className='h-3 w-3' />
                  </span>
                )}
              </button>
            )
          })}
        </div>
      </div>

      {/* Custom permissions picker */}
      {presetKey === 'custom' && (
        <div className='flex flex-col gap-4 rounded-md border border-border bg-muted/20 p-5'>
          <div className='flex items-center justify-between'>
            <div>
              <h3 className='text-sm font-semibold'>Custom permissions</h3>
              <p className='text-xs text-muted-foreground mt-0.5'>
                {customPerms.length === 0
                  ? 'Pick at least one permission to enable this role.'
                  : `${customPerms.length} permission${customPerms.length > 1 ? 's' : ''} selected.`}
              </p>
            </div>
          </div>
          <div className='flex flex-col gap-4'>
            {PERMISSION_GROUPS.map((group) => (
              <div key={group.label} className='flex flex-col gap-2'>
                <p className='text-[11px] font-semibold uppercase tracking-wider text-muted-foreground'>
                  {group.label}
                </p>
                <div className='flex flex-wrap gap-2'>
                  {group.permissions.map((perm) => {
                    const checked = customPerms.includes(perm)
                    return (
                      <button
                        key={perm}
                        type='button'
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
        </div>
      )}

      {/* Summary + actions */}
      <div className='flex items-center justify-between gap-3 pt-4 border-t border-border'>
        <p className='text-xs text-muted-foreground'>
          Will grant{' '}
          <span className='font-semibold text-foreground tabular-nums'>{effectivePerms.length}</span>{' '}
          permission{effectivePerms.length === 1 ? '' : 's'}.
        </p>
        <div className='flex items-center gap-2'>
          <button
            type='button'
            onClick={onCancel}
            className='rounded-md border border-border bg-background px-4 py-2 text-sm font-medium hover:bg-muted transition-colors'
          >
            Cancel
          </button>
          <button
            type='submit'
            disabled={submitDisabled}
            className='rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 transition-colors disabled:opacity-50 disabled:cursor-not-allowed'
          >
            {isSubmitting ? 'Creating…' : 'Create role'}
          </button>
        </div>
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
