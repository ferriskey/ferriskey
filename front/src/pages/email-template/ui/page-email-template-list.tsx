import { Skeleton } from '@/components/ui/skeleton'
import {
  ChevronDown,
  KeyRound,
  Link2,
  Mail,
  MailCheck,
  Pencil,
  Plus,
  Search,
  Trash2,
} from 'lucide-react'
import { useMemo, useState } from 'react'

interface EmailTemplate {
  id: string
  name: string
  email_type: string
  created_at: string
  updated_at: string
}

interface Props {
  templates: EmailTemplate[]
  isLoading: boolean
  onEdit: (id: string) => void
  onDelete: (id: string) => void
  onCreate: () => void
}

type EmailType = 'reset_password' | 'magic_link' | 'email_verification'
type TypeFilter = 'all' | EmailType
type SortKey = 'recent' | 'name'

const EMAIL_TYPES: { key: EmailType; label: string; short: string; icon: typeof Mail }[] = [
  { key: 'reset_password', label: 'Reset Password', short: 'Password', icon: KeyRound },
  { key: 'magic_link', label: 'Magic Link', short: 'Magic Link', icon: Link2 },
  { key: 'email_verification', label: 'Email Verification', short: 'Verification', icon: MailCheck },
]

const EMAIL_TONE: Record<EmailType, { bg: string; fg: string; border: string }> = {
  reset_password: {
    bg: 'bg-amber-500/10',
    fg: 'text-amber-500',
    border: 'border-amber-500/40',
  },
  magic_link: {
    bg: 'bg-violet-500/10',
    fg: 'text-violet-500',
    border: 'border-violet-500/40',
  },
  email_verification: {
    bg: 'bg-emerald-500/10',
    fg: 'text-emerald-500',
    border: 'border-emerald-500/40',
  },
}

const sortLabels: Record<SortKey, string> = {
  recent: 'Most recent',
  name: 'Name (A→Z)',
}

const formatRelative = (iso: string) => {
  const d = new Date(iso)
  if (Number.isNaN(d.getTime())) return '—'
  const diffMs = Date.now() - d.getTime()
  const sec = Math.round(diffMs / 1000)
  if (sec < 60) return 'just now'
  const min = Math.round(sec / 60)
  if (min < 60) return `${min}m ago`
  const hr = Math.round(min / 60)
  if (hr < 24) return `${hr}h ago`
  const days = Math.round(hr / 24)
  if (days < 30) return `${days}d ago`
  return d.toLocaleDateString()
}

const getEmailTypeMeta = (type: string) => {
  return EMAIL_TYPES.find((t) => t.key === type) ?? EMAIL_TYPES[0]
}

export default function PageEmailTemplateList({
  templates,
  isLoading,
  onEdit,
  onDelete,
  onCreate,
}: Props) {
  const [query, setQuery] = useState('')
  const [typeFilter, setTypeFilter] = useState<TypeFilter>('all')
  const [sort, setSort] = useState<SortKey>('recent')

  const stats = useMemo(() => {
    const total = templates.length
    const counts = EMAIL_TYPES.reduce<Record<EmailType, number>>(
      (acc, t) => {
        acc[t.key] = 0
        return acc
      },
      {} as Record<EmailType, number>,
    )
    for (const tpl of templates) {
      if (counts[tpl.email_type as EmailType] !== undefined) {
        counts[tpl.email_type as EmailType] += 1
      }
    }
    return { total, ...counts }
  }, [templates])

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase()
    let list = templates.filter((tpl) => {
      if (typeFilter !== 'all' && tpl.email_type !== typeFilter) return false
      if (!q) return true
      const hay = [tpl.name, tpl.email_type].filter(Boolean).join(' ').toLowerCase()
      return hay.includes(q)
    })
    list = [...list]
    if (sort === 'recent')
      list.sort((a, b) => new Date(b.created_at).getTime() - new Date(a.created_at).getTime())
    if (sort === 'name') list.sort((a, b) => a.name.localeCompare(b.name))
    return list
  }, [templates, query, typeFilter, sort])

  return (
    <div className='flex flex-col gap-6 p-4 sm:p-6 md:p-8 lg:p-12'>
      {/* Header */}
      <div className='flex flex-col gap-2 md:flex-row md:items-start md:justify-between'>
        <div>
          <h1 className='text-2xl font-medium tracking-tight'>Email Templates</h1>
          <p className='text-sm text-muted-foreground mt-1'>
            Customize transactional emails sent to your users: password resets, magic links, and
            verifications.
          </p>
        </div>
        <button
          onClick={onCreate}
          className='inline-flex items-center gap-2 rounded-md bg-primary px-3.5 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 transition-colors'
        >
          <Plus className='h-4 w-4' />
          New Template
        </button>
      </div>

      {/* Stats by type */}
      <div className='grid grid-cols-2 lg:grid-cols-4 gap-3'>
        <button
          onClick={() => setTypeFilter('all')}
          className={`flex items-center gap-3 rounded-md border bg-card/40 px-4 py-3 text-left transition ${
            typeFilter === 'all'
              ? 'border-primary ring-1 ring-primary/30'
              : 'border-border hover:border-primary/30'
          }`}
        >
          <div className='h-9 w-9 rounded-md bg-primary/10 flex items-center justify-center'>
            <Mail className='h-4 w-4 text-primary' />
          </div>
          <div>
            <div className='text-xl font-semibold tabular-nums leading-none'>
              {isLoading ? <Skeleton className='h-5 w-10' /> : stats.total}
            </div>
            <p className='text-xs text-muted-foreground mt-1'>All templates</p>
          </div>
        </button>
        {EMAIL_TYPES.map((t) => {
          const tone = EMAIL_TONE[t.key]
          const active = typeFilter === t.key
          return (
            <button
              key={t.key}
              onClick={() => setTypeFilter(t.key)}
              className={`flex items-center gap-3 rounded-md border bg-card/40 px-4 py-3 text-left transition ${
                active
                  ? `${tone.border} ring-1 ${tone.border.replace('border-', 'ring-')}`
                  : 'border-border hover:border-primary/30'
              }`}
            >
              <div className={`h-9 w-9 rounded-md flex items-center justify-center ${tone.bg}`}>
                <t.icon className={`h-4 w-4 ${tone.fg}`} />
              </div>
              <div>
                <div className='text-xl font-semibold tabular-nums leading-none'>
                  {isLoading ? <Skeleton className='h-5 w-10' /> : stats[t.key]}
                </div>
                <p className='text-xs text-muted-foreground mt-1 truncate'>{t.short}</p>
              </div>
            </button>
          )
        })}
      </div>

      {/* Toolbar */}
      <div className='flex flex-col gap-3 lg:flex-row lg:items-center lg:justify-between'>
        <div />
        <div className='flex items-center gap-2'>
          <div className='relative flex-1 sm:flex-none'>
            <Search className='pointer-events-none absolute left-3 top-1/2 -translate-y-1/2 h-3.5 w-3.5 text-muted-foreground' />
            <input
              type='search'
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              placeholder='Search templates…'
              className='w-full sm:w-64 rounded-md border border-border bg-background pl-9 pr-3 py-1.5 text-sm outline-none placeholder:text-muted-foreground focus:border-primary/40 focus:ring-1 focus:ring-primary/30'
            />
          </div>
          <div className='relative shrink-0'>
            <select
              value={sort}
              onChange={(e) => setSort(e.target.value as SortKey)}
              className='appearance-none rounded-md border border-border bg-background pl-3 pr-8 py-1.5 text-sm font-medium hover:bg-muted transition-colors cursor-pointer outline-none'
            >
              {(Object.keys(sortLabels) as SortKey[]).map((k) => (
                <option key={k} value={k}>
                  {sortLabels[k]}
                </option>
              ))}
            </select>
            <ChevronDown className='pointer-events-none absolute right-2 top-1/2 -translate-y-1/2 h-3.5 w-3.5 text-muted-foreground' />
          </div>
        </div>
      </div>

      {/* Grid */}
      {isLoading ? (
        <div className='grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-4'>
          {Array.from({ length: 6 }).map((_, i) => (
            <div
              key={`tpl-skel-${i}`}
              className='rounded-md border border-border bg-card/40 p-5 space-y-3'
            >
              <div className='flex items-center gap-3'>
                <Skeleton className='h-10 w-10 rounded-md' />
                <div className='space-y-2 flex-1'>
                  <Skeleton className='h-3 w-32' />
                  <Skeleton className='h-2 w-20' />
                </div>
              </div>
              <Skeleton className='h-3 w-full' />
            </div>
          ))}
        </div>
      ) : filtered.length === 0 ? (
        <div className='rounded-md border border-dashed border-border bg-muted/20 p-12 flex flex-col items-center justify-center text-center gap-3'>
          <div className='h-12 w-12 rounded-md bg-muted flex items-center justify-center'>
            <Mail className='h-6 w-6 text-muted-foreground' />
          </div>
          <div>
            <p className='text-sm font-medium'>
              {templates.length === 0
                ? 'Create your first email template'
                : 'No templates match your filters'}
            </p>
            <p className='text-xs text-muted-foreground mt-1'>
              {templates.length === 0
                ? 'Customize how transactional emails look when sent to your users.'
                : 'Adjust the search or type filter.'}
            </p>
          </div>
          {templates.length === 0 && (
            <button
              onClick={onCreate}
              className='inline-flex items-center gap-2 rounded-md bg-primary px-3.5 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 transition-colors mt-2'
            >
              <Plus className='h-4 w-4' />
              New Template
            </button>
          )}
        </div>
      ) : (
        <div className='grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-4'>
          {filtered.map((tpl) => {
            const meta = getEmailTypeMeta(tpl.email_type)
            const tone = EMAIL_TONE[tpl.email_type as EmailType] ?? EMAIL_TONE.reset_password
            return (
              <div
                key={tpl.id}
                className='group flex flex-col gap-4 rounded-md border border-border bg-card/40 p-5 text-left transition hover:border-primary/30 hover:bg-muted/40 hover:shadow-sm'
              >
                <div className='flex items-start gap-3'>
                  <div
                    className={`h-10 w-10 rounded-md flex items-center justify-center shrink-0 ${tone.bg}`}
                  >
                    <meta.icon className={`h-5 w-5 ${tone.fg}`} />
                  </div>
                  <div className='flex-1 min-w-0'>
                    <span className='text-sm font-medium truncate block'>{tpl.name}</span>
                    <p className='text-xs text-muted-foreground truncate'>{meta.label}</p>
                  </div>
                  <div className='flex items-center gap-1 shrink-0'>
                    <button
                      onClick={() => onEdit(tpl.id)}
                      className='p-1.5 rounded-md text-muted-foreground hover:text-foreground hover:bg-muted transition-colors'
                      title='Edit'
                    >
                      <Pencil className='h-3.5 w-3.5' />
                    </button>
                    <button
                      onClick={() => onDelete(tpl.id)}
                      className='p-1.5 rounded-md text-muted-foreground hover:text-destructive hover:bg-destructive/10 transition-colors'
                      title='Delete'
                    >
                      <Trash2 className='h-3.5 w-3.5' />
                    </button>
                  </div>
                </div>

                <div className='flex items-center gap-2 mt-auto pt-3 border-t border-border/60 text-xs'>
                  <span
                    className={`inline-flex items-center gap-1 rounded-md px-1.5 py-0.5 text-[10px] font-semibold uppercase tracking-wide ${tone.bg} ${tone.fg}`}
                  >
                    {meta.short}
                  </span>
                  <span className='ml-auto text-muted-foreground whitespace-nowrap'>
                    {formatRelative(tpl.updated_at || tpl.created_at)}
                  </span>
                </div>
              </div>
            )
          })}
        </div>
      )}
    </div>
  )
}
