import { Construction, type LucideIcon } from 'lucide-react'

interface Props {
  icon?: LucideIcon
  title: string
  description: string
  /** Bullet points describing what the tab will offer once shipped. */
  points?: string[]
}

export default function ComingSoonTab({ icon: Icon = Construction, title, description, points }: Props) {
  return (
    <div className='rounded-md border border-dashed border-border bg-muted/20 p-10 flex flex-col items-center text-center gap-4 max-w-xl'>
      <div className='h-12 w-12 rounded-md bg-muted flex items-center justify-center'>
        <Icon className='h-6 w-6 text-muted-foreground' />
      </div>
      <div>
        <div className='flex items-center justify-center gap-2'>
          <h2 className='text-base font-semibold'>{title}</h2>
          <span className='rounded-md bg-primary/10 px-1.5 py-0.5 text-[10px] font-semibold uppercase tracking-wide text-primary'>
            Coming soon
          </span>
        </div>
        <p className='text-sm text-muted-foreground mt-1'>{description}</p>
      </div>
      {points && points.length > 0 && (
        <ul className='text-left text-sm text-muted-foreground space-y-1.5'>
          {points.map((p) => (
            <li key={p} className='flex items-start gap-2'>
              <span className='mt-1.5 h-1 w-1 rounded-full bg-muted-foreground/50 shrink-0' />
              {p}
            </li>
          ))}
        </ul>
      )}
    </div>
  )
}
