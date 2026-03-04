import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { Switch } from '@/components/ui/switch'
import { Input } from '@/components/ui/input'
import { ConfigFieldDef } from '../constants/protocol-mapper-templates'

// ─── Single field ─────────────────────────────────────────────────────────────

interface MapperConfigFieldProps {
  fieldDef: ConfigFieldDef
  value: string
  onChange: (value: string) => void
}

export function MapperConfigField({ fieldDef, value, onChange }: MapperConfigFieldProps) {
  const fieldId = `mapper-config-${fieldDef.key.replace(/\./g, '-')}`

  if (fieldDef.type === 'switch') {
    return (
      <div className='flex items-center justify-between rounded-md border border-border px-3 py-2.5'>
        <div className='flex flex-col gap-0.5'>
          <span className='text-sm font-medium'>{fieldDef.label}</span>
          {fieldDef.description && (
            <span className='text-xs text-muted-foreground'>{fieldDef.description}</span>
          )}
        </div>
        <Switch
          checked={value === 'true'}
          onCheckedChange={(checked) => onChange(checked ? 'true' : 'false')}
        />
      </div>
    )
  }

  if (fieldDef.type === 'select') {
    return (
      <div className='flex flex-col gap-1.5'>
        <label htmlFor={fieldId} className='text-sm font-medium'>{fieldDef.label}</label>
        {fieldDef.description && (
          <p className='text-xs text-muted-foreground -mt-0.5'>{fieldDef.description}</p>
        )}
        <Select value={value} onValueChange={onChange}>
          <SelectTrigger className='w-full'>
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {fieldDef.options?.map((opt) => (
              <SelectItem key={opt.value} value={opt.value}>
                {opt.label}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </div>
    )
  }

  // text
  return (
    <div className='flex flex-col gap-1.5'>
      <label htmlFor={fieldId} className='text-sm font-medium'>{fieldDef.label}</label>
      {fieldDef.description && (
        <p className='text-xs text-muted-foreground -mt-0.5'>{fieldDef.description}</p>
      )}
      <Input
        id={fieldId}
        placeholder={fieldDef.placeholder}
        value={value}
        onChange={(e) => onChange(e.target.value)}
      />
    </div>
  )
}

// ─── Field group (text+select + switch section) ───────────────────────────────

interface MapperConfigFieldsProps {
  fields: ConfigFieldDef[]
  values: Record<string, string>
  onChange: (key: string, value: string) => void
}

export function MapperConfigFields({ fields, values, onChange }: MapperConfigFieldsProps) {
  const textFields = fields.filter((f) => f.type !== 'switch')
  const switchFields = fields.filter((f) => f.type === 'switch')

  return (
    <div className='flex flex-col gap-4'>
      {textFields.length > 0 && (
        <div className='flex flex-col gap-3'>
          {textFields.map((fieldDef) => (
            <MapperConfigField
              key={fieldDef.key}
              fieldDef={fieldDef}
              value={values[fieldDef.key] ?? fieldDef.defaultValue ?? ''}
              onChange={(val) => onChange(fieldDef.key, val)}
            />
          ))}
        </div>
      )}

      {switchFields.length > 0 && (
        <div className='flex flex-col gap-2'>
          <p className='text-xs font-semibold text-muted-foreground uppercase tracking-wide'>
            Token inclusion
          </p>
          <div className='flex flex-col gap-2'>
            {switchFields.map((fieldDef) => (
              <MapperConfigField
                key={fieldDef.key}
                fieldDef={fieldDef}
                value={values[fieldDef.key] ?? fieldDef.defaultValue ?? 'false'}
                onChange={(val) => onChange(fieldDef.key, val)}
              />
            ))}
          </div>
        </div>
      )}
    </div>
  )
}
