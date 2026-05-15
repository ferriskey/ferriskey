import { Badge } from '@/components/ui/badge'
import { Sparkles } from 'lucide-react'

interface Props {
  title: string
  description: string
}

export default function ConsoleComingSoon({ title, description }: Props) {
  return (
    <div className='flex flex-col gap-8 p-8 md:p-12'>
      <div className='flex flex-col gap-3'>
        <div className='flex items-center gap-3'>
          <h1 className='text-2xl font-medium tracking-tight'>{title}</h1>
          <Badge variant='outline' className='rounded-md gap-1.5'>
            <Sparkles className='h-3 w-3' />
            Coming soon
          </Badge>
        </div>
        <p className='text-sm text-muted-foreground max-w-2xl'>{description}</p>
      </div>

      <div className='rounded-md border border-dashed border-border bg-muted/20 p-12 flex flex-col items-center justify-center text-center gap-3'>
        <div className='h-12 w-12 rounded-md bg-primary/10 flex items-center justify-center'>
          <Sparkles className='h-6 w-6 text-primary' />
        </div>
        <p className='text-sm font-medium'>We&apos;re building this</p>
        <p className='text-xs text-muted-foreground max-w-md'>
          This view is part of the Console roadmap. Drop feedback in our discussions to shape what
          ships first.
        </p>
      </div>
    </div>
  )
}
