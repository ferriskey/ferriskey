import { Construction } from 'lucide-react'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'
import { EmptyState } from '@/components/kit'

export default function ConsoleNotBuiltYet({
  section,
  page,
}: {
  section: string
  page: string
}) {
  return (
    <div className={cn('mx-auto', tokens.page.maxWidth, tokens.page.padding)}>
      <EmptyState
        icon={Construction}
        label='Not built yet'
        hint={`The ${page.replace(/-/g, ' ')} screen of ${section.replace(/-/g, ' ')} lands here as the console is rebuilt.`}
      />
    </div>
  )
}
