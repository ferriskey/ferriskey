interface FieldProps {
  label: string
  value: string | undefined
  onChange: (value: string) => void
}

export function TextField({ label, value, onChange }: FieldProps) {
  return (
    <label className='flex flex-col gap-1 text-xs'>
      <span className='text-muted-foreground'>{label}</span>
      <input
        type='text'
        className='rounded border border-border bg-background px-2 py-1 text-sm'
        value={value ?? ''}
        onChange={(e) => onChange(e.target.value)}
      />
    </label>
  )
}

export function ColorField({ label, value, onChange }: FieldProps) {
  return (
    <label className='flex items-center justify-between gap-2 text-xs'>
      <span className='text-muted-foreground'>{label}</span>
      <div className='flex items-center gap-1'>
        <input
          type='color'
          className='h-6 w-6 cursor-pointer rounded border border-border'
          value={value ?? '#000000'}
          onChange={(e) => onChange(e.target.value)}
        />
        <input
          type='text'
          className='w-24 rounded border border-border bg-background px-1.5 py-0.5 text-xs'
          value={value ?? ''}
          onChange={(e) => onChange(e.target.value)}
        />
      </div>
    </label>
  )
}

interface SelectFieldProps extends Omit<FieldProps, 'onChange'> {
  options: { label: string; value: string }[]
  onChange: (value: string) => void
  allowEmpty?: boolean
}

export function SelectField({ label, value, options, onChange, allowEmpty = true }: SelectFieldProps) {
  return (
    <label className='flex flex-col gap-1 text-xs'>
      <span className='text-muted-foreground'>{label}</span>
      <select
        className='rounded border border-border bg-background px-2 py-1 text-sm'
        value={value ?? ''}
        onChange={(e) => onChange(e.target.value)}
      >
        {allowEmpty && <option value=''>—</option>}
        {options.map((opt) => (
          <option key={opt.value} value={opt.value}>
            {opt.label}
          </option>
        ))}
      </select>
    </label>
  )
}
