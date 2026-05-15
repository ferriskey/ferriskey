import { ApplicationType } from '@/routes/sub-router/applications.router'
import { ArrowLeft, ArrowRight, Boxes, Sparkles } from 'lucide-react'
import { useState } from 'react'
import { APPLICATION_TONE, APPLICATION_TYPES } from '../types'

interface Props {
  onCancel: () => void
  onPick: (type: ApplicationType) => void
}

export default function PageCreateApplicationType({ onCancel, onPick }: Props) {
  const [selected, setSelected] = useState<ApplicationType | null>(null)

  return (
    <div className='flex flex-col gap-8 p-8 md:p-12 max-w-4xl'>
      {/* Header */}
      <div>
        <button
          type='button'
          onClick={onCancel}
          className='inline-flex items-center gap-1.5 text-xs text-muted-foreground hover:text-foreground transition-colors mb-4'
        >
          <ArrowLeft className='h-3.5 w-3.5' />
          Back to applications
        </button>
        <div className='flex items-start gap-4'>
          <div className='h-12 w-12 rounded-md bg-primary/10 flex items-center justify-center'>
            <Boxes className='h-5 w-5 text-primary' />
          </div>
          <div>
            <h1 className='text-2xl font-medium tracking-tight'>What kind of application?</h1>
            <p className='text-sm text-muted-foreground mt-1'>
              We&apos;ll tailor the next step to match the auth flow your application needs.
            </p>
          </div>
        </div>
      </div>

      {/* Step indicator */}
      <div className='flex items-center gap-2 text-xs text-muted-foreground'>
        <span className='inline-flex items-center gap-1.5 rounded-md bg-primary/10 px-2 py-0.5 text-primary font-medium'>
          <Sparkles className='h-3 w-3' />
          Step 1 of 2
        </span>
        <span>Pick a type</span>
        <span className='text-muted-foreground/40'>·</span>
        <span className='text-muted-foreground/60'>Configure</span>
      </div>

      {/* Type cards */}
      <div className='grid grid-cols-1 sm:grid-cols-2 gap-4'>
        {APPLICATION_TYPES.map((t) => {
          const tone = APPLICATION_TONE[t.tone]
          const active = selected === t.key
          return (
            <button
              key={t.key}
              type='button'
              onClick={() => setSelected(t.key)}
              className={`relative flex flex-col items-start gap-4 rounded-md border bg-card/40 p-5 text-left transition ${
                active
                  ? `${tone.border} ring-2 ${tone.border.replace('border-', 'ring-')} bg-primary/[0.02]`
                  : 'border-border hover:border-primary/30 hover:bg-muted/40'
              }`}
            >
              <div className={`h-12 w-12 rounded-md flex items-center justify-center ${tone.bg}`}>
                <t.icon className={`h-6 w-6 ${tone.fg}`} />
              </div>
              <div>
                <p className='text-base font-medium'>{t.label}</p>
                <p className='text-xs text-muted-foreground mt-1'>{t.description}</p>
              </div>
              <div className='mt-auto flex items-center gap-2 text-[11px]'>
                <span className={`inline-flex items-center rounded-md px-1.5 py-0.5 font-semibold uppercase tracking-wide ${tone.bg} ${tone.fg}`}>
                  {t.short}
                </span>
                <span className='text-muted-foreground'>{t.flow}</span>
              </div>
            </button>
          )
        })}
      </div>

      {/* Actions */}
      <div className='flex items-center justify-between gap-3 pt-4 border-t border-border'>
        <p className='text-xs text-muted-foreground'>
          You can change the application type later, but some settings will reset.
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
            type='button'
            disabled={!selected}
            onClick={() => selected && onPick(selected)}
            className='inline-flex items-center gap-2 rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 transition-colors disabled:opacity-50 disabled:cursor-not-allowed'
          >
            Continue
            <ArrowRight className='h-4 w-4' />
          </button>
        </div>
      </div>
    </div>
  )
}
