import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { cn } from '@/lib/utils'

export interface SelectMenuOption {
  value: string
  label: string
  description?: string
}

export interface SelectMenuProps {
  options: SelectMenuOption[]
  value?: string
  onValueChange: (value: string) => void
  placeholder?: string
  disabled?: boolean
  className?: string
}

export function SelectMenu({
  options,
  value,
  onValueChange,
  placeholder = 'Select an option',
  disabled,
  className,
}: SelectMenuProps) {
  return (
    <Select value={value} onValueChange={onValueChange} disabled={disabled}>
      <SelectTrigger className={cn('w-full', className)}>
        <SelectValue placeholder={placeholder} />
      </SelectTrigger>
      <SelectContent>
        {options.map((option) => (
          <SelectItem key={option.value} value={option.value}>
            <div className='flex flex-col items-start gap-0.5'>
              <span>{option.label}</span>
              {option.description && (
                <span className='text-xs text-muted-foreground'>{option.description}</span>
              )}
            </div>
          </SelectItem>
        ))}
      </SelectContent>
    </Select>
  )
}

export default SelectMenu
