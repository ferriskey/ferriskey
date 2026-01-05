import { Check } from 'lucide-react'
import { cn } from '@/lib/utils'

export interface WizardStep {
  id: number
  title: string
  description?: string
}

interface ProviderWizardProgressProps {
  steps: WizardStep[]
  currentStep: number
}

export default function ProviderWizardProgress({
  steps,
  currentStep,
}: ProviderWizardProgressProps) {
  return (
    <nav aria-label='Progress' className='w-full'>
      <ol className='flex items-center'>
        {steps.map((step, stepIdx) => {
          const isCompleted = step.id < currentStep
          const isCurrent = step.id === currentStep
          const isPending = step.id > currentStep

          return (
            <li
              key={step.id}
              className={cn(
                'relative flex items-center',
                stepIdx !== steps.length - 1 && 'flex-1'
              )}
            >
              <div className='relative flex items-center gap-2'>
                {/* Step circle */}
                <span
                  className={cn(
                    'flex h-7 w-7 items-center justify-center rounded-full border-2 text-xs font-semibold transition-all duration-200 shrink-0',
                    isCompleted && 'bg-primary border-primary text-primary-foreground',
                    isCurrent && 'border-primary bg-primary text-primary-foreground ring-2 ring-primary/20',
                    isPending && 'border-muted-foreground/30 bg-background text-muted-foreground'
                  )}
                >
                  {isCompleted ? (
                    <Check className='h-3.5 w-3.5' />
                  ) : (
                    step.id
                  )}
                </span>

                {/* Step label */}
                <span
                  className={cn(
                    'text-sm font-medium transition-colors duration-200 whitespace-nowrap',
                    isCurrent && 'text-foreground',
                    isPending && 'text-muted-foreground',
                    isCompleted && 'text-foreground'
                  )}
                >
                  {step.title}
                </span>
              </div>

              {/* Connector line */}
              {stepIdx !== steps.length - 1 && (
                <div
                  className='flex-1 h-[2px] mx-3 transition-colors duration-200'
                  aria-hidden='true'
                >
                  <div
                    className={cn(
                      'h-full',
                      isCompleted ? 'bg-primary' : 'bg-muted'
                    )}
                  />
                </div>
              )}
            </li>
          )
        })}
      </ol>
    </nav>
  )
}

export const WIZARD_STEPS: WizardStep[] = [
  { id: 1, title: 'Select Provider' },
  { id: 2, title: 'Configure' },
  { id: 3, title: 'Review' },
]
