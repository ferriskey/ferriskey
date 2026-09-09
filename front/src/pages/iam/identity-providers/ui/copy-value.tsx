import { useState } from 'react'
import { Check, Copy } from 'lucide-react'
import { cn } from '@/lib/utils'

export interface CopyValueProps {
  value: string
  label: string
  className?: string
}

export default function CopyValue({ value, label, className }: CopyValueProps) {
  const [copied, setCopied] = useState(false)

  const copy = () => {
    void navigator.clipboard.writeText(value)
    setCopied(true)
    window.setTimeout(() => setCopied(false), 1500)
  }

  return (
    <div
      className={cn(
        'flex items-center gap-2 rounded-md border border-fk-line bg-neutral-50 dark:bg-fk-surface px-2.5 py-1.5',
        className
      )}
    >
      <code className='min-w-0 flex-1 truncate font-mono-ui text-xs text-neutral-700 dark:text-neutral-300'>
        {value}
      </code>
      <button
        type='button'
        aria-label={`Copy ${label}`}
        onClick={copy}
        className='shrink-0 cursor-pointer text-neutral-400 hover:text-neutral-700 dark:text-neutral-500 dark:hover:text-neutral-300'
      >
        {copied ? (
          <Check className='size-3.5 text-fk-success' />
        ) : (
          <Copy className='size-3.5' />
        )}
      </button>
    </div>
  )
}
