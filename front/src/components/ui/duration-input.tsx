import { useState } from 'react'
import { ChevronDownIcon } from 'lucide-react'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import {
  InputGroup,
  InputGroupAddon,
  InputGroupButton,
  InputGroupInput,
} from '@/components/ui/input-group'

type TimeUnit = 'seconds' | 'minutes' | 'hours' | 'days'

const MULTIPLIERS: Record<TimeUnit, number> = {
  seconds: 1,
  minutes: 60,
  hours: 3600,
  days: 86400,
}

const UNIT_LABELS: Record<TimeUnit, string> = {
  seconds: 'Seconds',
  minutes: 'Minutes',
  hours: 'Hours',
  days: 'Days',
}

function detectBestUnit(seconds: number): TimeUnit {
  if (seconds === 0) return 'seconds'
  if (seconds >= 86400 && seconds % 86400 === 0) return 'days'
  if (seconds >= 3600 && seconds % 3600 === 0) return 'hours'
  if (seconds >= 60 && seconds % 60 === 0) return 'minutes'
  return 'seconds'
}

interface DurationInputProps {
  label: string
  value: number | null
  onChange: (seconds: number | null) => void
  error?: string
  nullable?: boolean
}

export function DurationInput({
  label,
  value,
  onChange,
  error,
  nullable = false,
}: DurationInputProps) {
  const [userUnit, setUserUnit] = useState<TimeUnit | null>(null)

  const unit = userUnit ?? (value != null ? detectBestUnit(value) : 'seconds')

  const displayValue =
    value != null ? String(value / MULTIPLIERS[unit]) : ''

  const handleValueChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const raw = e.currentTarget.value
    if (raw === '') {
      onChange(nullable ? null : 0)
      return
    }
    const num = Number(raw)
    if (Number.isNaN(num)) return
    onChange(num * MULTIPLIERS[unit])
  }

  const handleUnitChange = (newUnit: TimeUnit) => {
    setUserUnit(newUnit)
    if (value != null) {
      const currentDisplay = value / MULTIPLIERS[unit]
      onChange(currentDisplay * MULTIPLIERS[newUnit])
    }
  }

  return (
    <div>
      <InputGroup className={error ? 'border-destructive ring-destructive/20' : ''}>
        <InputGroupInput
          type='number'
          placeholder={label}
          value={displayValue}
          onChange={handleValueChange}
        />
        <InputGroupAddon align='inline-end'>
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <InputGroupButton variant='ghost' className='pr-1.5! text-xs'>
                {UNIT_LABELS[unit]} <ChevronDownIcon className='size-3' />
              </InputGroupButton>
            </DropdownMenuTrigger>
            <DropdownMenuContent align='end'>
              <DropdownMenuGroup>
                {(Object.keys(MULTIPLIERS) as TimeUnit[]).map((u) => (
                  <DropdownMenuItem key={u} onSelect={() => handleUnitChange(u)}>
                    {UNIT_LABELS[u]}
                  </DropdownMenuItem>
                ))}
              </DropdownMenuGroup>
            </DropdownMenuContent>
          </DropdownMenu>
        </InputGroupAddon>
      </InputGroup>

      {error && (
        <p className='mt-0.5 px-3 text-xs font-medium text-destructive'>{error}</p>
      )}
    </div>
  )
}
