import type { BuilderNode } from '../../builder-core'

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
          className='w-20 rounded border border-border bg-background px-1.5 py-0.5 text-xs'
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
}

export function SelectField({
  label,
  value,
  options,
  onChange,
}: SelectFieldProps) {
  return (
    <label className='flex flex-col gap-1 text-xs'>
      <span className='text-muted-foreground'>{label}</span>
      <select
        className='rounded border border-border bg-background px-2 py-1 text-sm'
        value={value ?? ''}
        onChange={(e) => onChange(e.target.value)}
      >
        <option value=''>—</option>
        {options.map((opt) => (
          <option key={opt.value} value={opt.value}>
            {opt.label}
          </option>
        ))}
      </select>
    </label>
  )
}

interface PaddingFieldsProps {
  node: BuilderNode
  onUpdate: (
    updates: Partial<Pick<BuilderNode, 'props' | 'styles'>>,
  ) => void
}

export function PaddingFields({ node, onUpdate }: PaddingFieldsProps) {
  const update = (key: string, value: string) => {
    onUpdate({ props: { [key]: value } })
  }

  return (
    <div className='flex flex-col gap-1'>
      <span className='text-xs text-muted-foreground'>Padding</span>
      <div className='grid grid-cols-2 gap-1'>
        <TextField
          label='Top'
          value={node.props['padding-top'] as string}
          onChange={(v) => update('padding-top', v)}
        />
        <TextField
          label='Bottom'
          value={node.props['padding-bottom'] as string}
          onChange={(v) => update('padding-bottom', v)}
        />
        <TextField
          label='Left'
          value={node.props['padding-left'] as string}
          onChange={(v) => update('padding-left', v)}
        />
        <TextField
          label='Right'
          value={node.props['padding-right'] as string}
          onChange={(v) => update('padding-right', v)}
        />
      </div>
    </div>
  )
}
