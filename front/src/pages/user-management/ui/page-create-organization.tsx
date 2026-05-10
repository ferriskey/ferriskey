import { ArrowLeft, Building2, Globe } from 'lucide-react'
import { useEffect } from 'react'
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import { z } from 'zod'

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

const createOrganizationSchema = z.object({
  name: z.string().trim().min(1, 'Name is required'),
  alias: z
    .string()
    .trim()
    .min(1, 'Alias is required')
    .regex(/^[a-z0-9_-]+$/, 'Only lowercase letters, numbers, hyphens and underscores.'),
  domain: z.string().trim().optional().default(''),
  description: z.string().trim().optional().default(''),
})

type CreateOrganizationSchema = z.infer<typeof createOrganizationSchema>

export default function PageCreateOrganization({ onCancel, onSubmit, isSubmitting }: Props) {
  const {
    register,
    handleSubmit,
    watch,
    setValue,
    formState: { errors, isValid, dirtyFields },
  } = useForm<CreateOrganizationSchema>({
    resolver: zodResolver(createOrganizationSchema),
    defaultValues: { name: '', alias: '', domain: '', description: '' },
    mode: 'onChange',
  })

  const name = watch('name')
  const alias = watch('alias')
  const aliasDirty = Boolean(dirtyFields.alias)

  // Auto-derive alias from name until the user manually edits the alias field.
  useEffect(() => {
    if (aliasDirty) return
    setValue('alias', slugify(name ?? ''), { shouldValidate: true })
  }, [name, aliasDirty, setValue])

  const submit = handleSubmit((values) => {
    onSubmit({
      name: values.name.trim(),
      alias: values.alias.trim(),
      domain: (values.domain ?? '').trim(),
      description: (values.description ?? '').trim(),
    })
  })

  const submitDisabled = !isValid || isSubmitting

  return (
    <form onSubmit={submit} className='flex flex-col gap-8 p-8 md:p-12 max-w-2xl'>
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
          error={errors.name?.message}
        >
          <input
            type='text'
            {...register('name')}
            placeholder='Acme Inc.'
            autoFocus
            className='w-full rounded-md border border-border bg-background px-3 py-2 text-sm outline-none placeholder:text-muted-foreground focus:border-primary/40 focus:ring-1 focus:ring-primary/30'
          />
        </Field>

        <Field
          label='Alias'
          required
          hint='Lowercase identifier used in URLs and APIs. Auto-derived from the name.'
          error={errors.alias?.message && (alias?.length ?? 0) > 0 ? errors.alias.message : undefined}
        >
          <input
            type='text'
            {...register('alias')}
            placeholder='acme'
            className='w-full rounded-md border border-border bg-background px-3 py-2 text-sm font-mono outline-none placeholder:text-muted-foreground focus:border-primary/40 focus:ring-1 focus:ring-primary/30'
          />
        </Field>

        <Field
          label='Domain'
          optional
          hint='Optional. Used to auto-route users with matching email to this organization.'
          error={errors.domain?.message}
        >
          <div className='relative'>
            <Globe className='pointer-events-none absolute left-3 top-1/2 -translate-y-1/2 h-3.5 w-3.5 text-muted-foreground' />
            <input
              type='text'
              {...register('domain')}
              placeholder='acme.com'
              className='w-full rounded-md border border-border bg-background pl-9 pr-3 py-2 text-sm outline-none placeholder:text-muted-foreground focus:border-primary/40 focus:ring-1 focus:ring-primary/30'
            />
          </div>
        </Field>

        <Field
          label='Description'
          optional
          hint='Internal note to remember what this organization is about.'
          error={errors.description?.message}
        >
          <textarea
            {...register('description')}
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
          disabled={submitDisabled}
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
