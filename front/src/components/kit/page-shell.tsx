import type { ReactNode } from 'react'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'

export function PageShell({ children, className }: { children: ReactNode; className?: string }) {
  return <div className={cn('mx-auto', tokens.page.maxWidth, tokens.page.padding, className)}>{children}</div>
}
