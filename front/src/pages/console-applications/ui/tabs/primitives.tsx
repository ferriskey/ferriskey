import { Check, Copy } from 'lucide-react'
import { useState } from 'react'

interface SectionProps {
  title: string
  description?: string
  tone?: 'default' | 'danger'
  children: React.ReactNode
}

export function Section({ title, description, tone = 'default', children }: SectionProps) {
  return (
    <section
      className={`rounded-md border bg-card/40 p-5 flex flex-col gap-4 ${
        tone === 'danger' ? 'border-red-500/30' : 'border-border'
      }`}
    >
      <div>
        <h2 className='text-sm font-semibold'>{title}</h2>
        {description && <p className='text-xs text-muted-foreground mt-0.5'>{description}</p>}
      </div>
      {children}
    </section>
  )
}

interface FieldProps {
  label: string
  hint?: string
  children: React.ReactNode
}

export function Field({ label, hint, children }: FieldProps) {
  return (
    <label className='flex flex-col gap-1.5'>
      <span className='text-sm font-medium'>{label}</span>
      {children}
      {hint && <p className='text-xs text-muted-foreground'>{hint}</p>}
    </label>
  )
}

/** Read-only value with a copy-to-clipboard affordance. */
export function CopyRow({ value, mono = true }: { value: string; mono?: boolean }) {
  const [copied, setCopied] = useState(false)
  const copy = () => {
    void navigator.clipboard.writeText(value)
    setCopied(true)
    window.setTimeout(() => setCopied(false), 1500)
  }
  return (
    <div className='flex items-center gap-2'>
      <input
        readOnly
        value={value}
        className={`flex-1 rounded-md border border-border bg-muted/40 px-3 py-2 text-sm outline-none ${mono ? 'font-mono' : ''}`}
      />
      <button
        type='button'
        onClick={copy}
        className='inline-flex h-9 w-9 items-center justify-center rounded-md border border-border bg-background text-muted-foreground hover:text-foreground hover:bg-muted transition-colors shrink-0'
        aria-label='Copy'
      >
        {copied ? <Check className='h-3.5 w-3.5 text-emerald-500' /> : <Copy className='h-3.5 w-3.5' />}
      </button>
    </div>
  )
}
