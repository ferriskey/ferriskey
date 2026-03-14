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
  // Unit is set once at mount, then only changed via dropdown or external reset
  const [unit, setUnit] = useState<TimeUnit>(() =>
    value != null ? detectBestUnit(value) : 'seconds'
  )

  // Local display string for the input — avoids recalculating from value each render
  const [displayStr, setDisplayStr] = useState(() =>
    value != null ? String(value / MULTIPLIERS[detectBestUnit(value)]) : ''
  )

  // Track previous prop value and last emitted value to distinguish
  // internal changes (our onChange) from external ones (form reset)
  const [prevValue, setPrevValue] = useState(value)
  const [lastEmitted, setLastEmitted] = useState(value)

  if (prevValue !== value) {
    setPrevValue(value)
    // External change: value differs from what we last emitted
    if (value !== lastEmitted) {
      const newUnit = detectBestUnit(value ?? 0)
      setUnit(newUnit)
      setDisplayStr(value != null ? String(value / MULTIPLIERS[newUnit]) : '')
      setLastEmitted(value)
    }
  }

  const handleValueChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const raw = e.currentTarget.value
    setDisplayStr(raw)

    if (raw === '') {
      const emitted = nullable ? null : 0
      setLastEmitted(emitted)
      onChange(emitted)
      return
    }
    const num = Number(raw)
    if (!Number.isNaN(num)) {
      const emitted = num * MULTIPLIERS[unit]
      setLastEmitted(emitted)
      onChange(emitted)
    }
  }

  const handleUnitChange = (newUnit: TimeUnit) => {
    setUnit(newUnit)
    if (displayStr !== '') {
      const currentDisplay = Number(displayStr) || 0
      const emitted = currentDisplay * MULTIPLIERS[newUnit]
      setLastEmitted(emitted)
      onChange(emitted)
    }
  }

  return (
    <div>
      <InputGroup className={error ? 'border-destructive ring-destructive/20' : ''}>
        <InputGroupInput
          type='number'
          placeholder={label}
          value={displayStr}
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
