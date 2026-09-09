import { AlertTriangle, CheckCircle2, XCircle } from 'lucide-react'
import { Pill, type PillTone } from '@/components/kit'
import type { ProviderHealth } from '../provider-status'

const tones: Record<ProviderHealth, PillTone> = {
  healthy: 'success',
  degraded: 'amber',
  error: 'danger',
}

const icons = {
  healthy: CheckCircle2,
  degraded: AlertTriangle,
  error: XCircle,
} as const

export interface ProviderStatusPillProps {
  health: ProviderHealth
  label: string
}

export default function ProviderStatusPill({ health, label }: ProviderStatusPillProps) {
  const Icon = icons[health]

  return (
    <Pill tone={tones[health]} mono>
      <Icon className='size-3' strokeWidth={2} />
      {label}
    </Pill>
  )
}
